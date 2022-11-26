// Most of this code is taken from https://github.com/qdot/systray-rs/blob/master/src/api/win32/mod.rs

mod funcs;
mod structs;

use std::{
    cell::RefCell,
    mem,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread,
};

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::{
            Shell::{
                Shell_NotifyIconW, NIF_ICON, NIF_TIP, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
            },
            WindowsAndMessaging::{
                InsertMenuItemW, LoadImageW, PostMessageW, HICON, IMAGE_ICON, LR_DEFAULTCOLOR,
                MENUITEMINFOW, MFS_DISABLED, MFS_UNHILITE, MFT_SEPARATOR, MFT_STRING, MIIM_FTYPE,
                MIIM_ID, MIIM_STATE, MIIM_STRING, WM_DESTROY,
            },
        },
    },
};

use crate::TIError;
use funcs::*;
use structs::*;

thread_local!(static WININFO_STASH: RefCell<Option<WindowsLoopData>> = RefCell::new(None));

type CallBackEntry = Option<Box<dyn Fn() + Send + 'static>>;

pub struct TrayItemWindows {
    entries: Arc<Mutex<Vec<CallBackEntry>>>,
    info: WindowInfo,
    windows_loop: Option<thread::JoinHandle<()>>,
    event_loop: Option<thread::JoinHandle<()>>,
    event_tx: Sender<WindowsTrayEvent>,
}

impl TrayItemWindows {
    pub fn new(title: &str, icon: &str) -> Result<Self, TIError> {
        let entries = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel();
        let (event_tx, event_rx) = channel::<WindowsTrayEvent>();

        let entries_clone = Arc::clone(&entries);
        let event_loop = thread::spawn(move || loop {
            if let Ok(v) = event_rx.recv() {
                if v.0 == u32::MAX {
                    break;
                }

                padlock::mutex_lock(&entries_clone, |ents: &mut Vec<CallBackEntry>| match &ents
                    [v.0 as usize]
                {
                    Some(f) => f(),
                    None => (),
                })
            }
        });

        let event_tx_clone = event_tx.clone();
        let windows_loop = thread::spawn(move || unsafe {
            let info = match init_window() {
                Ok(info) => {
                    tx.send(Ok(info.clone())).ok();
                    info
                }

                Err(e) => {
                    tx.send(Err(e)).ok();
                    return;
                }
            };

            WININFO_STASH.with(|stash| {
                let data = WindowsLoopData {
                    info,
                    tx: event_tx_clone,
                };

                (*stash.borrow_mut()) = Some(data);
            });

            run_loop();
        });

        let info = match rx.recv().unwrap() {
            Ok(i) => i,
            Err(e) => return Err(e),
        };

        let w = Self {
            entries,
            info,
            windows_loop: Some(windows_loop),
            event_loop: Some(event_loop),
            event_tx,
        };

        w.set_tooltip(title)?;
        w.set_icon(icon)?;

        Ok(w)
    }

    pub fn set_icon(&self, icon: &str) -> Result<(), TIError> {
        self.set_icon_from_resource(icon)
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(None);
            len
        }) as u32;

        let mut st = to_wstring(label);
        let item = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE,
            fType: MFT_STRING,
            fState: MFS_DISABLED | MFS_UNHILITE,
            wID: item_idx,
            dwTypeData: PWSTR::from_raw(st.as_mut_ptr()),
            cch: (label.len() * 2) as u32,
            ..Default::default()
        };
        unsafe {
            if !InsertMenuItemW(self.info.hmenu, item_idx, true, &item).as_bool() {
                return Err(get_win_os_error("Error inserting menu item"));
            }
        }
        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() + Send + 'static,
    {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(Some(Box::new(cb)));
            len
        }) as u32;

        let mut st = to_wstring(label);
        let item = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE,
            fType: MFT_STRING,
            wID: item_idx,
            dwTypeData: PWSTR::from_raw(st.as_mut_ptr()),
            cch: (label.len() * 2) as u32,
            ..Default::default()
        };
        unsafe {
            if !InsertMenuItemW(self.info.hmenu, item_idx, true, &item).as_bool() {
                return Err(get_win_os_error("Error inserting menu item"));
            }
        }
        Ok(())
    }

    pub fn add_separator(&mut self) -> Result<(), TIError> {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(None);
            len
        }) as u32;
        let item = MENUITEMINFOW {
            cbSize: mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_ID | MIIM_STATE,
            fType: MFT_SEPARATOR,
            wID: item_idx,
            ..Default::default()
        };
        unsafe {
            if !InsertMenuItemW(self.info.hmenu, item_idx, true, &item).as_bool() {
                return Err(get_win_os_error("Error inserting menu separator"));
            }
        }
        Ok(())
    }

    // others

    fn set_tooltip(&self, tooltip: &str) -> Result<(), TIError> {
        // Add Tooltip
        // Gross way to convert String to [i8; 128]
        // TODO: Clean up conversion, test for length so we don't panic at runtime
        let mut nid = NOTIFYICONDATAW {
            cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.info.hwnd,
            uID: 1,
            uFlags: NIF_TIP,
            ..Default::default()
        };
        for (index, character) in to_wstring(tooltip).into_iter().enumerate() {
            nid.szTip[index] = character;
        }
        unsafe {
            if !Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool() {
                return Err(get_win_os_error("Error setting tooltip"));
            }
        }
        Ok(())
    }

    fn set_icon_from_resource(&self, resource_name: &str) -> Result<(), TIError> {
        let icon = unsafe {
            match LoadImageW(
                self.info.hinstance,
                PCWSTR::from_raw(to_wstring(resource_name).as_ptr()),
                IMAGE_ICON,
                64,
                64,
                LR_DEFAULTCOLOR,
            ) {
                Ok(handle) => HICON(handle.0),
                Err(_) => return Err(get_win_os_error("Error setting icon from resource")),
            }
        };
        self._set_icon(icon)
    }

    fn _set_icon(&self, icon: HICON) -> Result<(), TIError> {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.info.hwnd,
                uID: 1,
                uFlags: NIF_ICON,
                hIcon: icon,
                ..Default::default()
            };
            if !Shell_NotifyIconW(NIM_MODIFY, &nid).as_bool() {
                return Err(get_win_os_error("Error setting icon"));
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        unsafe {
            PostMessageW(self.info.hwnd, WM_DESTROY, WPARAM(0), LPARAM(0));
        }
        if let Some(t) = self.windows_loop.take() {
            t.join().ok();
        }
        if let Some(t) = self.event_loop.take() {
            self.event_tx.send(WindowsTrayEvent(u32::MAX)).ok();
            t.join().ok();
        }
    }

    pub fn shutdown(&self) -> Result<(), TIError> {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.info.hwnd,
                uID: 1,
                uFlags: NIF_ICON,
                ..Default::default()
            };
            if !Shell_NotifyIconW(NIM_DELETE, &nid).as_bool() {
                return Err(get_win_os_error("Error deleting icon from menu"));
            }
        }

        Ok(())
    }
}

impl Drop for TrayItemWindows {
    fn drop(&mut self) {
        self.shutdown().ok();
        self.quit();
    }
}

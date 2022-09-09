// Most of this code is taken from https://github.com/qdot/systray-rs/blob/master/src/api/win32/mod.rs

use {
    crate::TIError,
    std::{
        self,
        cell::RefCell,
        sync::{
            mpsc::{channel, Sender},
            Arc, Mutex,
        },
        thread,
    },
    winapi::{
        shared::{
            minwindef::{LPARAM, WPARAM},
            windef::HICON,
        },
        um::{
            shellapi::{self, NIF_ICON, NIF_TIP, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW},
            winuser::{
                self, IMAGE_ICON, MENUITEMINFOW, MFS_DISABLED, MFS_UNHILITE, MFT_SEPARATOR,
                MFT_STRING, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, WM_DESTROY,
            },
        },
    },
};

mod funcs;
mod structs;
use funcs::*;
use structs::*;

thread_local!(static WININFO_STASH: RefCell<Option<WindowsLoopData>> = RefCell::new(None));

type CallBackEntry = Option<Box<dyn Fn() -> () + Send + 'static>>;

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
            match event_rx.recv() {
                Ok(v) => {
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

                Err(_) => (),
            }
        });

        let event_tx_clone = event_tx.clone();
        let windows_loop = thread::spawn(move || unsafe {
            let i = init_window();
            let k;

            match i {
                Ok(j) => {
                    tx.send(Ok(j.clone())).ok();
                    k = j;
                }

                Err(e) => {
                    tx.send(Err(e)).ok();
                    return;
                }
            }

            WININFO_STASH.with(|stash| {
                let data = WindowsLoopData {
                    info: k,
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
            entries: entries,
            info: info,
            windows_loop: Some(windows_loop),
            event_loop: Some(event_loop),
            event_tx: event_tx,
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
        let mut item = get_menu_item_struct();
        item.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE;
        item.fType = MFT_STRING;
        item.fState = MFS_DISABLED | MFS_UNHILITE;
        item.wID = item_idx;
        item.dwTypeData = st.as_mut_ptr();
        item.cch = (label.len() * 2) as u32;
        unsafe {
            if winuser::InsertMenuItemW(self.info.hmenu, item_idx, 1, &item as *const MENUITEMINFOW)
                == 0
            {
                return Err(get_win_os_error("Error inserting menu item"));
            }
        }
        Ok(())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), TIError> {
        panic!("TODO");
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() -> () + Send + 'static,
    {
        let item_idx = padlock::mutex_lock(&self.entries, |entries| {
            let len = entries.len();
            entries.push(Some(Box::new(cb)));
            len
        }) as u32;

        let mut st = to_wstring(label);
        let mut item = get_menu_item_struct();
        item.fMask = MIIM_FTYPE | MIIM_STRING | MIIM_ID | MIIM_STATE;
        item.fType = MFT_STRING;
        item.wID = item_idx;
        item.dwTypeData = st.as_mut_ptr();
        item.cch = (label.len() * 2) as u32;
        unsafe {
            if winuser::InsertMenuItemW(self.info.hmenu, item_idx, 1, &item as *const MENUITEMINFOW)
                == 0
            {
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
        let mut item = get_menu_item_struct();
        item.fMask = MIIM_FTYPE | MIIM_ID | MIIM_STATE;
        item.fType = MFT_SEPARATOR;
        item.wID = item_idx;
        unsafe {
            if winuser::InsertMenuItemW(self.info.hmenu, item_idx, 1, &item as *const MENUITEMINFOW)
                == 0
            {
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
        let tt = tooltip.as_bytes().clone();
        let mut nid = get_nid_struct(&self.info.hwnd);
        for i in 0..tt.len() {
            nid.szTip[i] = tt[i] as u16;
        }
        nid.uFlags = NIF_TIP;
        unsafe {
            if shellapi::Shell_NotifyIconW(NIM_MODIFY, &mut nid as *mut NOTIFYICONDATAW) == 0 {
                return Err(get_win_os_error("Error setting tooltip"));
            }
        }
        Ok(())
    }

    fn set_icon_from_resource(&self, resource_name: &str) -> Result<(), TIError> {
        let icon;
        unsafe {
            icon = winuser::LoadImageW(
                self.info.hinstance,
                to_wstring(&resource_name).as_ptr(),
                IMAGE_ICON,
                64,
                64,
                0,
            ) as HICON;
            if icon == std::ptr::null_mut() as HICON {
                return Err(get_win_os_error("Error setting icon from resource"));
            }
        }
        self._set_icon(icon)
    }

    fn _set_icon(&self, icon: HICON) -> Result<(), TIError> {
        unsafe {
            let mut nid = get_nid_struct(&self.info.hwnd);
            nid.uFlags = NIF_ICON;
            nid.hIcon = icon;
            if shellapi::Shell_NotifyIconW(NIM_MODIFY, &mut nid as *mut NOTIFYICONDATAW) == 0 {
                return Err(get_win_os_error("Error setting icon"));
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) {
        unsafe {
            winuser::PostMessageW(self.info.hwnd, WM_DESTROY, 0 as WPARAM, 0 as LPARAM);
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
            let mut nid = get_nid_struct(&self.info.hwnd);
            nid.uFlags = NIF_ICON;
            if shellapi::Shell_NotifyIconW(NIM_DELETE, &mut nid as *mut NOTIFYICONDATAW) == 0 {
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

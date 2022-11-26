use std::{ffi::OsStr, mem, os::windows::ffi::OsStrExt};

use windows::Win32::{
    Foundation::{GetLastError, HINSTANCE, HWND, LRESULT, POINT},
    Graphics::Gdi::HBRUSH,
    System::LibraryLoader::GetModuleHandleA,
    UI::{
        Shell::{NIF_MESSAGE, NIM_ADD},
        WindowsAndMessaging::{
            CreatePopupMenu, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos,
            GetMenuItemID, GetMessageW, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassW,
            SetForegroundWindow, SetMenuInfo, TrackPopupMenu, TranslateMessage, CW_USEDEFAULT,
            IDI_APPLICATION, MENUINFO, MIM_APPLYTOSUBMENUS, MIM_STYLE, MNS_NOTIFYBYPOS, MSG,
            TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_LEFTBUTTON, WINDOW_EX_STYLE, WM_LBUTTONUP,
            WM_MENUCOMMAND, WM_QUIT, WM_RBUTTONUP, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use {super::*, crate::TIError};

pub(crate) fn to_wstring(str: &str) -> Vec<u16> {
    OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

pub(crate) unsafe fn get_win_os_error(msg: &str) -> TIError {
    TIError::new_with_location(
        format!("{}: {}", &msg, GetLastError().0),
        std::file!(),
        std::line!(),
    )
}

pub(crate) unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_MENUCOMMAND {
        WININFO_STASH.with(|stash| {
            let stash = stash.borrow();
            let stash = stash.as_ref();
            if let Some(stash) = stash {
                let menu_id = GetMenuItemID(stash.info.hmenu, w_param.0 as _) as i32;
                if menu_id != -1 {
                    stash.tx.send(WindowsTrayEvent(menu_id as u32)).ok();
                }
            }
        });
    }

    if msg == WM_USER + 1 && (l_param.0 as u32 == WM_LBUTTONUP || l_param.0 as u32 == WM_RBUTTONUP)
    {
        let mut p = POINT { x: 0, y: 0 };
        if !GetCursorPos(&mut p as *mut POINT).as_bool() {
            return LRESULT(1);
        }
        SetForegroundWindow(h_wnd);
        WININFO_STASH.with(|stash| {
            let stash = stash.borrow();
            let stash = stash.as_ref();
            if let Some(stash) = stash {
                TrackPopupMenu(
                    stash.info.hmenu,
                    TPM_LEFTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                    p.x,
                    p.y,
                    0,
                    h_wnd,
                    None,
                );
            }
        });
    }
    if msg == WM_DESTROY {
        PostQuitMessage(0);
    }
    DefWindowProcW(h_wnd, msg, w_param, l_param)
}

pub(crate) unsafe fn init_window() -> Result<WindowInfo, TIError> {
    let class_name = to_wstring("my_window");
    let hinstance = GetModuleHandleA(None).unwrap(); // FG
    let wnd = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hIcon: LoadIconW(HINSTANCE::default(), IDI_APPLICATION).unwrap(), // FG
        hCursor: LoadCursorW(HINSTANCE::default(), IDI_APPLICATION).unwrap(), // FG
        hbrBackground: HBRUSH(16),
        lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
        ..Default::default()
    };
    if RegisterClassW(&wnd) == 0 {
        return Err(get_win_os_error("Error creating window class"));
    }
    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE(0),
        PCWSTR::from_raw(class_name.as_ptr()),
        PCWSTR::from_raw(to_wstring("rust_systray_window").as_ptr()),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        0,
        CW_USEDEFAULT,
        0,
        None,
        None,
        None,
        None,
    );
    if hwnd.0 == 0 {
        return Err(get_win_os_error("Error creating window"));
    }
    let mut nid = NOTIFYICONDATAW {
        cbSize: mem::size_of::<NOTIFYICONDATAW>() as _,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE,
        uCallbackMessage: WM_USER + 1,
        ..Default::default()
    };
    if !Shell_NotifyIconW(NIM_ADD, &mut nid as *mut NOTIFYICONDATAW).as_bool() {
        return Err(get_win_os_error("Error adding menu icon"));
    }
    // Setup menu
    let hmenu = CreatePopupMenu().unwrap(); // FG
    let m = MENUINFO {
        cbSize: mem::size_of::<MENUINFO>() as _,
        fMask: MIM_APPLYTOSUBMENUS | MIM_STYLE,
        dwStyle: MNS_NOTIFYBYPOS,
        ..Default::default()
    };
    if !SetMenuInfo(hmenu, &m).as_bool() {
        return Err(get_win_os_error("Error setting up menu"));
    }

    Ok(WindowInfo {
        hwnd,
        hmenu,
        hinstance,
    })
}

pub(crate) unsafe fn run_loop() {
    // Run message loop
    let mut msg = MSG::default();
    loop {
        GetMessageW(&mut msg, None, 0, 0);
        if msg.message == WM_QUIT {
            break;
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

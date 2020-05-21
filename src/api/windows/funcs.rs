use super::*;
use crate::TIError;
use std::{
    self,
    ffi::OsStr,
    os::windows::ffi::OsStrExt
};
use winapi::{
    ctypes::{c_ulong, c_ushort},
    shared::{
        basetsd::ULONG_PTR,
        guiddef::GUID,
        minwindef::{DWORD, HINSTANCE, LPARAM, LRESULT, UINT, WPARAM},
        ntdef::LPCWSTR,
        windef::{HBITMAP, HBRUSH, HICON, HMENU, HWND, POINT},
    },
    um::{
        errhandlingapi, libloaderapi,
        shellapi::{self, NIF_MESSAGE, NIM_ADD, NOTIFYICONDATAW},
        winuser::{
            self, CW_USEDEFAULT, MENUINFO, MENUITEMINFOW,
            MIM_APPLYTOSUBMENUS, MIM_STYLE, MNS_NOTIFYBYPOS, WM_USER, WNDCLASSW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

pub(crate) fn to_wstring(str: &str) -> Vec<u16> {
    OsStr::new(str)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

pub(crate) unsafe fn get_win_os_error(msg: &str) -> TIError {
    TIError::new_with_location(format!("{}: {}", &msg, errhandlingapi::GetLastError()), std::file!(), std::line!())
}

pub(crate) unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == winuser::WM_MENUCOMMAND {
        WININFO_STASH.with(|stash| {
            let stash = stash.borrow();
            let stash = stash.as_ref();
            if let Some(stash) = stash {
                let menu_id = winuser::GetMenuItemID(stash.info.hmenu, w_param as i32) as i32;
                if menu_id != -1 {
                    stash
                        .tx
                        .send(WindowsTrayEvent(menu_id as u32))
                        .ok();
                }
            }
        });
    }

    if msg == WM_USER + 1 {
        if l_param as UINT == winuser::WM_LBUTTONUP || l_param as UINT == winuser::WM_RBUTTONUP {
            let mut p = POINT { x: 0, y: 0 };
            if winuser::GetCursorPos(&mut p as *mut POINT) == 0 {
                return 1;
            }
            winuser::SetForegroundWindow(h_wnd);
            WININFO_STASH.with(|stash| {
                let stash = stash.borrow();
                let stash = stash.as_ref();
                if let Some(stash) = stash {
                    winuser::TrackPopupMenu(
                        stash.info.hmenu,
                        0,
                        p.x,
                        p.y,
                        (winuser::TPM_BOTTOMALIGN | winuser::TPM_LEFTALIGN) as i32,
                        h_wnd,
                        std::ptr::null_mut(),
                    );
                }
            });
        }
    }
    if msg == winuser::WM_DESTROY {
        winuser::PostQuitMessage(0);
    }
    return winuser::DefWindowProcW(h_wnd, msg, w_param, l_param);
}

pub(crate) fn get_nid_struct(hwnd: &HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
        hWnd: *hwnd,
        uID: 0x1 as UINT,
        uFlags: 0 as UINT,
        uCallbackMessage: 0 as UINT,
        hIcon: 0 as HICON,
        szTip: [0 as u16; 128],
        dwState: 0 as DWORD,
        dwStateMask: 0 as DWORD,
        szInfo: [0 as u16; 256],
        u: Default::default(),
        szInfoTitle: [0 as u16; 64],
        dwInfoFlags: 0 as UINT,
        guidItem: GUID {
            Data1: 0 as c_ulong,
            Data2: 0 as c_ushort,
            Data3: 0 as c_ushort,
            Data4: [0; 8],
        },
        hBalloonIcon: 0 as HICON,
    }
}

pub(crate) fn get_menu_item_struct() -> MENUITEMINFOW {
    MENUITEMINFOW {
        cbSize: std::mem::size_of::<MENUITEMINFOW>() as UINT,
        fMask: 0 as UINT,
        fType: 0 as UINT,
        fState: 0 as UINT,
        wID: 0 as UINT,
        hSubMenu: 0 as HMENU,
        hbmpChecked: 0 as HBITMAP,
        hbmpUnchecked: 0 as HBITMAP,
        dwItemData: 0 as ULONG_PTR,
        dwTypeData: std::ptr::null_mut(),
        cch: 0 as u32,
        hbmpItem: 0 as HBITMAP,
    }
}

pub(crate) unsafe fn init_window() -> Result<WindowInfo, TIError> {
    let class_name = to_wstring("my_window");
    let hinstance: HINSTANCE = libloaderapi::GetModuleHandleA(std::ptr::null_mut());
    let wnd = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: 0 as HINSTANCE,
        hIcon: winuser::LoadIconW(0 as HINSTANCE, winuser::IDI_APPLICATION),
        hCursor: winuser::LoadCursorW(0 as HINSTANCE, winuser::IDI_APPLICATION),
        hbrBackground: 16 as HBRUSH,
        lpszMenuName: 0 as LPCWSTR,
        lpszClassName: class_name.as_ptr(),
    };
    if winuser::RegisterClassW(&wnd) == 0 {
        return Err(get_win_os_error("Error creating window class"));
    }
    let hwnd = winuser::CreateWindowExW(
        0,
        class_name.as_ptr(),
        to_wstring("rust_systray_window").as_ptr(),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        0,
        CW_USEDEFAULT,
        0,
        0 as HWND,
        0 as HMENU,
        0 as HINSTANCE,
        std::ptr::null_mut(),
    );
    if hwnd == std::ptr::null_mut() {
        return Err(get_win_os_error("Error creating window"));
    }
    let mut nid = get_nid_struct(&hwnd);
    nid.uID = 0x1;
    nid.uFlags = NIF_MESSAGE;
    nid.uCallbackMessage = WM_USER + 1;
    if shellapi::Shell_NotifyIconW(NIM_ADD, &mut nid as *mut NOTIFYICONDATAW) == 0 {
        return Err(get_win_os_error("Error adding menu icon"));
    }
    // Setup menu
    let hmenu = winuser::CreatePopupMenu();
    let m = MENUINFO {
        cbSize: std::mem::size_of::<MENUINFO>() as DWORD,
        fMask: MIM_APPLYTOSUBMENUS | MIM_STYLE,
        dwStyle: MNS_NOTIFYBYPOS,
        cyMax: 0 as UINT,
        hbrBack: 0 as HBRUSH,
        dwContextHelpID: 0 as DWORD,
        dwMenuData: 0 as ULONG_PTR,
    };
    if winuser::SetMenuInfo(hmenu, &m as *const MENUINFO) == 0 {
        return Err(get_win_os_error("Error setting up menu"));
    }

    Ok(WindowInfo {
        hwnd: hwnd,
        hmenu: hmenu,
        hinstance: hinstance,
    })
}

pub(crate) unsafe fn run_loop() {
    // Run message loop
    let mut msg = winuser::MSG {
        hwnd: 0 as HWND,
        message: 0 as UINT,
        wParam: 0 as WPARAM,
        lParam: 0 as LPARAM,
        time: 0 as DWORD,
        pt: POINT { x: 0, y: 0 },
    };
    loop {
        winuser::GetMessageW(&mut msg, 0 as HWND, 0, 0);
        if msg.message == winuser::WM_QUIT {
            break;
        }
        winuser::TranslateMessage(&mut msg);
        winuser::DispatchMessageW(&mut msg);
    }
}

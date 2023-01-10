use std::sync::mpsc::Sender;

use windows_sys::Win32::{
    Foundation::{HINSTANCE, HWND},
    UI::WindowsAndMessaging::HMENU,
};

#[derive(Clone)]
pub(crate) struct WindowInfo {
    pub hwnd: HWND,
    pub hinstance: HINSTANCE,
    pub hmenu: HMENU,
}

unsafe impl Send for WindowInfo {}
unsafe impl Sync for WindowInfo {}

#[derive(Clone)]
pub(crate) struct WindowsLoopData {
    pub info: WindowInfo,
    pub tx: Sender<WindowsTrayEvent>,
}

pub(crate) struct WindowsTrayEvent(pub(crate) u32);

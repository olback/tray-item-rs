#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Set type depending on OS
#[cfg(target_os = "linux")]
pub type TrayItemImpl = linux::TrayItemLinux;

#[cfg(target_os = "windows")]
pub type TrayItemImpl = windows::TrayItemWindows;

#[cfg(target_os = "macos")]
pub type TrayItemImpl = macos::TrayItemMacOS;

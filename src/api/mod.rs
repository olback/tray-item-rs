#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Set type depending on OS
#[cfg(target_os = "linux")]
pub(crate) type TrayIndicatorImpl = linux::TrayIndicatorLinux;

#[cfg(target_os = "windows")]
pub(crate) type TrayIndicatorImpl = windows::TrayIndicatorWindows;

#[cfg(target_os = "macos")]
pub(crate) type TrayIndicatorImpl = macos::TrayIndicatorMacOS;

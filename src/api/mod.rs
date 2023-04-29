#[cfg(all(target_os = "linux", feature = "ksni"))]
mod linux_ksni;

#[cfg(all(target_os = "linux", feature = "libappindicator"))]
mod linux_libappindicator;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Set type depending on OS and feature
#[cfg(all(target_os = "linux", feature = "ksni"))]
pub type TrayItemImpl = linux_ksni::TrayItemLinux;

#[cfg(all(target_os = "linux", feature = "libappindicator"))]
pub type TrayItemImpl = linux_libappindicator::TrayItemLinux;

#[cfg(target_os = "windows")]
pub type TrayItemImpl = windows::TrayItemWindows;

#[cfg(target_os = "macos")]
pub type TrayItemImpl = macos::TrayItemMacOS;

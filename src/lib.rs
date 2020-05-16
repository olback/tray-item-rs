// Imports
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Set type depending on OS
#[cfg(target_os = "linux")]
type TrayIndicatorImpl = linux::TrayIndicatorLinux;

#[cfg(target_os = "windows")]
type TrayIndicatorImpl = windows::TrayIndicatorWindows;

#[cfg(target_os = "macos")]
type TrayIndicatorImpl = macos::TrayIndicatorMacOS;


pub struct TrayIndicator(TrayIndicatorImpl);

impl TrayIndicator {

    pub fn new(title: &str, icon: &str) -> Self {

        Self(TrayIndicatorImpl::new(title, icon))

    }

    pub fn show(&mut self) {

        self.0.show()

    }

    pub fn hide(&mut self) {

        self.0.hide()

    }

    pub fn add_label(&mut self, label: &str) {

        self.0.add_label(label);

    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F)
        where F: Fn(&gtk::MenuItem) -> () + 'static {

       self.0.add_menu_item(label, cb);

    }

    pub fn add_divider(&mut self) {

        self.0.add_divider();

    }

}

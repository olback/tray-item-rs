use libappindicator::{AppIndicator, AppIndicatorStatus};
use gtk::prelude::*;

pub(crate) struct TrayIndicatorLinux {
    tray: AppIndicator,
    menu: gtk::Menu
}

impl TrayIndicatorLinux {

    pub(crate) fn new(title: &str, icon: &str) -> Self {

        let tray = AppIndicator::new(title, icon);

        Self {
            tray: tray,
            menu: gtk::Menu::new()
        }

    }

    pub(crate) fn set_attention_icon(&mut self, icon: &str) {

        self.tray.set_attention_icon(icon)

    }

    pub(crate) fn show(&mut self, attention: bool) {

        self.tray.set_status(if attention {
            AppIndicatorStatus::Attention
        } else {
            AppIndicatorStatus::Active
        });

    }

    pub(crate) fn hide(&mut self) {

        self.tray.set_status(AppIndicatorStatus::Passive);

    }

    pub(crate) fn add_label(&mut self, label: &str) {

        let item = gtk::MenuItem::new_with_label(label.as_ref());
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

    }

    pub(crate) fn add_menu_item<F>(&mut self, label: &str, cb: F)
        where F: Fn(&gtk::MenuItem) -> () + 'static {

        let item = gtk::MenuItem::new_with_label(label.as_ref());
        item.connect_activate(cb);
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

    }

}

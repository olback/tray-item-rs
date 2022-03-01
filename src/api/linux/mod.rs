use {
    crate::TIError,
    gtk::prelude::*,
    libappindicator::{AppIndicator, AppIndicatorStatus},
};

pub struct TrayItemLinux {
    tray: AppIndicator,
    menu: gtk::Menu,
}

impl TrayItemLinux {
    pub fn new(title: &str, icon: &str) -> Result<Self, TIError> {
        let mut t = Self {
            tray: AppIndicator::new(title, icon),
            menu: gtk::Menu::new(),
        };

        t.set_icon(icon)?;

        Ok(t)
    }

    pub fn set_icon(&mut self, icon: &str) -> Result<(), TIError> {
        self.tray.set_icon(icon);
        self.tray.set_status(AppIndicatorStatus::Active);

        Ok(())
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        let item = gtk::MenuItem::with_label(label.as_ref());
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

        Ok(())
    }

    pub fn set_label(&mut self, label: &str) -> Result<(), TIError> {
        panic!("TODO");
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() -> () + Send + 'static,
    {
        let item = gtk::MenuItem::with_label(label.as_ref());
        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

        Ok(())
    }
}

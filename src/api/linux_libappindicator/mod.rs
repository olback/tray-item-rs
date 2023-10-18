use {
    crate::{IconSource, TIError},
    gtk::prelude::*,
    libappindicator::{AppIndicator, AppIndicatorStatus},
};

pub struct TrayItemLinux {
    tray: AppIndicator,
    menu: gtk::Menu,
}

impl TrayItemLinux {
    pub fn new(title: &str, icon: IconSource) -> Result<Self, TIError> {
        let mut t = Self {
            tray: AppIndicator::new(title, icon.as_str()),
            menu: gtk::Menu::new(),
        };

        t.set_icon(icon)?;

        Ok(t)
    }

    pub fn set_icon(&mut self, icon: IconSource) -> Result<(), TIError> {
        self.tray.set_icon(icon.as_str());
        self.tray.set_status(AppIndicatorStatus::Active);

        Ok(())
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        let item = gtk::MenuItem::with_label(label);
        item.set_sensitive(false);
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() + Send + 'static,
    {
        let item = gtk::MenuItem::with_label(label);
        item.connect_activate(move |_| {
            cb();
        });
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

        Ok(())
    }

    pub fn add_separator(&mut self) -> Result<(), TIError> {
        let item = gtk::SeparatorMenuItem::new();
        self.menu.append(&item);
        self.menu.show_all();
        self.tray.set_menu(&mut self.menu);

        Ok(())
    }
}

use cocoa::{
    base::id,
    foundation::{NSData, NSSize},
};

use {
    crate::IconSource,
    crate::TIError,
    callback::*,
    cocoa::{
        appkit::{
            NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps, NSImage, NSMenu,
            NSMenuItem, NSRunningApplication, NSStatusBar, NSStatusItem, NSWindow,
        },
        base::{nil, YES},
        foundation::{NSAutoreleasePool, NSString},
    },
    objc::{msg_send, sel, sel_impl},
    std::thread::JoinHandle,
};

mod callback;

fn get_icon_image(icon: IconSource) -> Option<id> {
    unsafe {
        match icon {
            IconSource::Resource(_) => {
                let icon = icon.as_str();
                let icon = Some(icon).filter(|icon| !icon.is_empty());
                icon.map(|icon_name| {
                    let icon_name = NSString::alloc(nil).init_str(icon_name);
                    NSImage::imageNamed_(NSImage::alloc(nil), icon_name)
                })
            }
            IconSource::Data {
                height,
                width,
                data,
            } => {
                let data = NSData::dataWithBytes_length_(
                    nil,
                    data.as_ptr() as *const std::os::raw::c_void,
                    data.len() as u64,
                );
                let image = NSImage::initWithData_(NSImage::alloc(nil), data);
                let new_size = if width != 0 && height != 0 {
                    let icon_height: f64 = 18.0;
                    let icon_width: f64 = (width as f64) / (height as f64 / icon_height);
                    NSSize::new(icon_width, icon_height)
                } else {
                    NSSize::new(18.0, 18.0)
                };
                let _: () = msg_send![image, setSize: new_size];
                Some(image)
            }
        }
    }
}

pub struct TrayItemMacOS {
    name: String,
    menu: *mut objc::runtime::Object,
    _pool: *mut objc::runtime::Object,
    icon: Option<*mut objc::runtime::Object>,
    main_thread: Option<JoinHandle<()>>,
}

impl TrayItemMacOS {
    pub fn new(title: &str, icon: IconSource) -> Result<Self, TIError> {
        let t = unsafe {
            let pool = NSAutoreleasePool::new(nil);

            TrayItemMacOS {
                name: title.to_string(),
                _pool: pool,
                icon: get_icon_image(icon),
                menu: NSMenu::new(nil).autorelease(),
                main_thread: None,
            }
        };
        Ok(t)
    }

    pub fn set_icon(&mut self, icon: IconSource) -> Result<(), TIError> {
        self.icon = get_icon_image(icon);
        Ok(())
    }

    pub fn set_icon_template(&mut self, icon: &str) -> Result<(), TIError> {
        unsafe {
            let icon_name = NSString::alloc(nil).init_str(icon);
            let image = NSImage::imageNamed_(NSImage::alloc(nil), icon_name);
            let _: () = msg_send![image, setTemplate: YES];
            self.icon = Some(image);
        }
        Ok(())
    }

    pub fn add_label(&mut self, label: &str) -> Result<(), TIError> {
        unsafe {
            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually
            let itemtitle = NSString::alloc(nil).init_str(label);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            let _: () = msg_send![item, setTitle: itemtitle];

            NSMenu::addItem_(self.menu, item);
        }

        Ok(())
    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F) -> Result<(), TIError>
    where
        F: Fn() -> () + Send + 'static,
    {
        let cb_obj = Callback::from(Box::new(cb));

        unsafe {
            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually
            let itemtitle = NSString::alloc(nil).init_str(label);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            let _: () = msg_send![item, setTarget: cb_obj];

            NSMenu::addItem_(self.menu, item);
        }

        Ok(())
    }

    // private

    pub fn add_quit_item(&mut self, label: &str) {
        unsafe {
            let no_key = NSString::alloc(nil).init_str("");
            let pref_item = NSString::alloc(nil).init_str(label);
            let pref_action = sel!(terminate:);
            let menuitem = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                pref_item,
                pref_action,
                no_key,
            );

            self.menu.addItem_(menuitem);
        }
    }

    pub fn display(&mut self) {
        unsafe {
            let app = NSApp();
            app.activateIgnoringOtherApps_(YES);

            let item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(-1.0);
            let title = NSString::alloc(nil).init_str(&self.name);
            if let Some(icon) = self.icon {
                let _: () = msg_send![item, setImage: icon];
            } else {
                item.setTitle_(title);
            }
            item.setMenu_(self.menu);

            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

            app.run();
        }
    }
}

impl Drop for TrayItemMacOS {
    fn drop(&mut self) {
        match self.main_thread.take() {
            Some(t) => t.join(),
            None => Ok(()),
        }
        .unwrap()
    }
}

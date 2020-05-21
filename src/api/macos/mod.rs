use crate::TIError;
use std::{
    mem,
    thread::{self, JoinHandle}
};
use objc::{
    sel,
    sel_impl,
    msg_send,
    Message,
    declare::ClassDecl,
    runtime::{Class, Object, Sel}
};
use objc_id::Id;
use objc_foundation::{INSObject, NSObject};
use cocoa::{
    base::{nil, YES},
    appkit::{
        NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps, NSMenu, NSMenuItem,
        NSRunningApplication, NSStatusBar, NSStatusItem, NSWindow
    },
    foundation::{NSAutoreleasePool, NSString}
};

mod callback;
use callback::*;

pub struct TrayItemMacOS {
    name: String,
    menu: *mut objc::runtime::Object,
    pool: *mut objc::runtime::Object,
    main_thread: Option<JoinHandle<()>>
}

impl TrayItemMacOS {

    pub fn new(title: &str, icon: &str) -> Result<Self, TIError> {

        unsafe {

            let pool = NSAutoreleasePool::new(nil);

            let mut t = TrayItemMacOS {
                name: title.to_string(),
                pool: pool,
                menu: NSMenu::new(nil).autorelease(),
                main_thread: None
            };

            // t.display();

            Ok(t)

        }

    }

    pub fn set_icon(&self, icon: &str) -> Result<(), TIError> {

        todo!()

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
        where F: Fn() -> () + Send + Sync + 'static {

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
            item.setTitle_(title);
            item.setMenu_(self.menu);

            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

            app.run();

        }

    }

}

struct SendWrapper(pub *mut objc::runtime::Object);
unsafe impl Send for SendWrapper {}

impl Drop for TrayItemMacOS {
    fn drop(&mut self) {
        match self.main_thread.take() {
            Some(t) => t.join(),
            None => Ok(())
        }.unwrap()
    }
}

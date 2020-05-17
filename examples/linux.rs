use tray_indicator::TrayIndicator;
use gtk;

fn main() {

    gtk::init().unwrap();

    let mut tray = TrayIndicator::new("Tray Example", "accessories-calculator");

    tray.add_label("Tray Label");

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    });

    tray.add_menu_item("Quit", || {
        gtk::main_quit();
    });

    tray.show(false);

    gtk::main();

}

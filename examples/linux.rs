use tray_item::TrayItem;
use gtk;

fn main() {

    gtk::init().unwrap();

    let mut tray = TrayItem::new("Tray Example", "accessories-calculator").unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    }).unwrap();

    tray.add_menu_item("Quit", || {
        gtk::main_quit();
    }).unwrap();

    gtk::main();

}

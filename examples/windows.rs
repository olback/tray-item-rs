use tray_item::{TrayItem, IconSource};

fn main() {

    let mut tray = TrayItem::new("Tray Example", IconSource::Resource("name-of-icon-in-rc-file")).unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    }).unwrap();

    tray.add_menu_item("Quit", || {
        println!("Quit");
        std::process::exit(0);
    }).unwrap();

    tray.set_icon(IconSource::Resource("another-name-from-rc-file")).unwrap();

    std::io::stdin().read_line(&mut String::new()).unwrap();

}

use tray_item::TrayItem;

fn main() {
    let mut tray = TrayItem::new("Tray Example", "my-icon-name").unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();

    tray.add_menu_item("Quit", || {
        println!("Quit");
        std::process::exit(0);
    })
    .unwrap();

    std::io::stdin().read_line(&mut String::new()).unwrap();
}

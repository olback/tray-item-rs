use tray_item::TrayItem;

fn main() {

    let mut tray = TrayItem::new("Tray Example", "").unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    }).unwrap();

    let mut inner = tray.inner_mut();
    inner.add_quit_item("Quit");
    inner.display();

}

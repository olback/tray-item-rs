use {std::sync::mpsc, tray_item::TrayItem, IconSource};

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new("Tray Example", IconSource::Resource("name-of-icon-in-rc-file")).unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();

    let (tx, rx) = mpsc::channel();

    tray.add_menu_item("Quit", move || {
        println!("Quit");
        tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    tray.set_icon(IconSource::Resource("another-name-from-rc-file")).unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => break,
            _ => {}
        }
    }
}

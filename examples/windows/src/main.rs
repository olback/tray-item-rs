use {std::sync::mpsc, tray_item::TrayItem};

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new("Tray Example", "my-icon-name").unwrap();

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

    loop {
        match rx.recv() {
            Ok(Message::Quit) => break,
            _ => {}
        }
    }
}

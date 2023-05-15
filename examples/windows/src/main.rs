use std::sync::{Arc, Mutex};
use {
    std::sync::mpsc,
    tray_item::{IconSource, TrayItem},
};

enum Message {
    Quit,
    Green,
    Red,
}

fn main() {
    let mut tray = TrayItem::new(
        "Tray Example",
        IconSource::Resource("name-of-icon-in-rc-file"),
    )
    .unwrap();

    tray.add_label("Tray Label").unwrap();

    tray.add_menu_item("Hello", || {
        println!("Hello!");
    })
    .unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::channel();

    let arc_tx = Arc::new(Mutex::new(tx));

    let red_tx = Arc::clone(&arc_tx);
    tray.add_menu_item("Red", move || {
        red_tx.lock().unwrap().send(Message::Red).unwrap();
    })
    .unwrap();

    let green_tx = Arc::clone(&arc_tx);
    tray.add_menu_item("Green", move || {
        green_tx.lock().unwrap().send(Message::Green).unwrap();
    })
    .unwrap();

    tray.inner_mut().add_separator().unwrap();

    let quit_tx = Arc::clone(&arc_tx);
    tray.add_menu_item("Quit", move || {
        quit_tx.lock().unwrap().send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::Red) => {
                println!("Red");
                tray.set_icon(IconSource::Resource("another-name-from-rc-file"))
                    .unwrap();
            }
            Ok(Message::Green) => {
                println!("Green");
                tray.set_icon(IconSource::Resource("name-of-icon-in-rc-file"))
                    .unwrap()
            }
            _ => {}
        }
    }
}

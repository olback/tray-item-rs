use std::sync::mpsc;
use tray_item::{IconSource, TrayItem};

enum Message {
    Quit,
    ChangeIcon,
    Hello,
}

enum Icon {
    Red,
    Green,
}

impl Icon {
    fn resource(&self) -> IconSource {
        match self {
            Self::Red => IconSource::Resource("another-name-from-rc-file"),
            Self::Green => IconSource::Resource("name-of-icon-in-rc-file"),
        }
    }
}

fn main() {
    let mut tray = TrayItem::new(
        "Tray Example",
        Icon::Green.resource(),
    )
    .unwrap();

    let label_id = tray.inner_mut().add_label_with_id("Tray Label").unwrap();

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let hello_tx = tx.clone();
    tray.add_menu_item("Hello!", move || {
        hello_tx.send(Message::Hello).unwrap();
    })
    .unwrap();

    let color_tx = tx.clone();
    let color_id = tray.inner_mut().add_menu_item_with_id("Change to Red", move || {
        color_tx.send(Message::ChangeIcon).unwrap();
    })
    .unwrap();
    let mut current_icon = Icon::Green;

    tray.inner_mut().add_separator().unwrap();

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            Ok(Message::ChangeIcon) => {
                let (next_icon, next_message) = match current_icon {
                    Icon::Red => (Icon::Green, "Change to Red"),
                    Icon::Green => (Icon::Red, "Change to Green"),
                };
                current_icon = next_icon;

                tray.inner_mut().set_menu_item_label(next_message, color_id).unwrap();
                tray.set_icon(current_icon.resource())
                    .unwrap();
            },
            Ok(Message::Hello) => {
                tray.inner_mut().set_label("Hi there!", label_id).unwrap();
            },
            _ => {}
        }
    }
}

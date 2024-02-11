use {std::io::Cursor, std::sync::mpsc, tray_item::IconSource, tray_item::TrayItem};

enum Message {
    Quit,
    Update,
}

fn main() {
    let cursor_red = Cursor::new(include_bytes!("../../resources/tray_icon-red.png"));
    let decoder_red = png::Decoder::new(cursor_red);
    let (info_red, mut reader_red) = decoder_red.read_info().unwrap();
    let mut buf_red = vec![0; info_red.buffer_size()];
    reader_red.next_frame(&mut buf_red).unwrap();

    let icon_red = IconSource::Data {
        data: buf_red,
        height: 32,
        width: 32,
    };

    let mut tray = TrayItem::new("Tray Example", icon_red).unwrap();

    tray.add_label("Tray Label").unwrap();

    let (tx, rx) = mpsc::sync_channel::<Message>(2);
    let update_tx = tx.clone();
    let id_menu = tray
        .inner_mut()
        .add_menu_item_with_id("Update Menu Item", move || {
            update_tx.send(Message::Update).unwrap();
        })
        .unwrap();

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
            Ok(Message::Update) => {
                println!("Update Menu Item!");
                tray.inner_mut()
                    .set_menu_item_label("Menu Updated", id_menu)
                    .unwrap();
            }
            _ => {}
        }
    }
}

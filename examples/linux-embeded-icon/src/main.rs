use {
    std::io::Cursor,
    std::sync::mpsc,
    tray_item::TrayItem,
    tray_item::IconSource
};

enum Message {
    Quit,
    Red,
    Green
}

fn main() {
    let cursor_red = Cursor::new(include_bytes!("../../resources/tray_icon-red.png"));
    let decoder_red = png::Decoder::new(cursor_red);
    let (info_red, mut reader_red) = decoder_red.read_info().unwrap();
    let mut buf_red = vec![0;info_red.buffer_size()];
    reader_red.next_frame(&mut buf_red).unwrap();

    let icon_red = IconSource::Data{data: buf_red, height: 32, width: 32};

    let mut tray = TrayItem::new("Tray Example", icon_red).unwrap();

    tray.add_label("Tray Label").unwrap();

    let (tx, rx) = mpsc::sync_channel::<Message>(2);
    let green_tx = tx.clone();
    tray.add_menu_item("Set icon green", move || {
        green_tx.send(Message::Green).unwrap();
    })
    .unwrap();

    let red_tx = tx.clone();
    tray.add_menu_item("Set icon red", move || {
        red_tx.send(Message::Red).unwrap();
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
                break
            },
            Ok(Message::Green) =>{
                println!("Green!");
                let cursor_green = Cursor::new(include_bytes!("../../resources/tray_icon-green.png"));
                let decoder_green = png::Decoder::new(cursor_green);
                let (info_green, mut reader_green) = decoder_green.read_info().unwrap();
                let mut buf_green = vec![0;info_green.buffer_size()];
                reader_green.next_frame(&mut buf_green).unwrap();
                let icon_green = IconSource::Data{data: buf_green, height: 32, width: 32};
                tray.set_icon(icon_green).unwrap();
            },
            Ok(Message::Red) => {
                println!("Red!");
                let cursor_red = Cursor::new(include_bytes!("../../resources/tray_icon-red.png"));
                let decoder_red = png::Decoder::new(cursor_red);
                let (info_red, mut reader_red) = decoder_red.read_info().unwrap();
                let mut buf_red = vec![0;info_red.buffer_size()];
                reader_red.next_frame(&mut buf_red).unwrap();
                let icon_red = IconSource::Data{data: buf_red, height: 32, width: 32};
                tray.set_icon(icon_red).unwrap();
            },
            _ => {}
        }
    }

}

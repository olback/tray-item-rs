use {
    gio::ResourceLookupFlags,
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
    // gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");
    let res_bytes = include_bytes!("../target/debug/build/linux-embeded-icon-e8cf0183594f6150/out/compiled.gresource");
    let data = gtk::glib::Bytes::from(&res_bytes[..]);
    let resource = gio::Resource::from_data(&data).unwrap();
    gio::resources_register(&resource);
    let children = resource.enumerate_children("/", ResourceLookupFlags::all());
    print!("{:#?}", children);

    let png = gio::resources_lookup_data("/name-of-icon-in-rc-file", ResourceLookupFlags::all()).expect("Failed to load png");
    println!("png size: {}", png.len());

    let mut tray = TrayItem::new("Tray Example", IconSource::Resource("/name-of-icon-in-rc-file")).unwrap();

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
                println!("Quit!");
                break
            },
            Ok(Message::Green) => {
                println!("Green!");
                tray.set_icon(IconSource::Resource("/another-name-from-rc-file")).unwrap();
            },
            Ok(Message::Red) => {
                println!("Red!");
                tray.set_icon(IconSource::Resource("/name-of-icon-in-rc-file")).unwrap();
            },
            _ => {}
        }
    }

}

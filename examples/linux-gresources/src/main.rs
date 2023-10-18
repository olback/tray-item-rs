use {
    gio::ResourceLookupFlags, std::sync::mpsc, std::thread, tray_item::IconSource,
    tray_item::TrayItem,
};

enum Message {
    Quit,
    NOP,
    Red,
    Green,
}

fn main() {
    gtk::init().unwrap();

    // gio::resources_register_include!("compiled.gresource").expect("Failed to register resources.");
    let res_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/compiled.gresource"));
    let data = gtk::glib::Bytes::from(&res_bytes[..]);
    let resource = gio::Resource::from_data(&data).unwrap();
    gio::resources_register(&resource);
    let children = resource.enumerate_children("/", ResourceLookupFlags::all());
    print!("{:#?}", children);

    let png = gio::resources_lookup_data("/name-of-icon-in-rc-file", ResourceLookupFlags::all())
        .expect("Failed to load png");
    println!("png size: {}", png.len());

    let mut tray = TrayItem::new(
        "Tray Example",
        IconSource::Resource("/name-of-icon-in-rc-file"),
    )
    .unwrap();

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

    glib::idle_add_local(move || match rx.recv() {
        Ok(Message::Quit) => {
            gtk::main_quit();
            println!("Quit!");
            glib::ControlFlow::Break
        }
        Ok(Message::Green) => {
            println!("Green!");
            tray.set_icon(IconSource::Resource("/another-name-from-rc-file"))
                .unwrap();
            glib::ControlFlow::Continue
        }
        Ok(Message::Red) => {
            println!("Red!");
            tray.set_icon(IconSource::Resource("/name-of-icon-in-rc-file"))
                .unwrap();
            glib::ControlFlow::Continue
        }
        _ => {
            println!("Default!");
            glib::ControlFlow::Continue
        }
    });

    thread::spawn(move || {
        let mut count = 0;
        loop {
            // Menu doesn't show up until after hitting enter a few times?
            //let mut s = String::new();
            //std::io::stdin().read_line(&mut s).unwrap();
            //if s.as_bytes()[0] == b'q' {
            //    println!("stopping thread loop!");
            //    break
            //}

            // glib::idle_add_local doesn't loop without this?
            count += 1;
            thread::sleep(std::time::Duration::from_millis(10));
            if count % 100 == 0 {
                tx.send(Message::NOP).unwrap();
                println!("Idle loop, {}!", count);
            }
        }
    });
    gtk::main();
}

use windres::Build;

fn main() {
    Build::new().compile("tray-example.rc").unwrap();
}

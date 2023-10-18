fn main() {
    glib_build_tools::compile_resources(
        &["../resources"],
        "../resources/tray-icon.xml",
        "compiled.gresource",
    );
}

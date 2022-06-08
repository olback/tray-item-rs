extern crate gio;

fn main() {
	gio::compile_resources(
		"../resources",
		"../resources/tray-icon.xml",
		"compiled.gresource",
	);
}

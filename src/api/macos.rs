pub struct TrayIndicatorMacOS;

impl TrayIndicatorMacOS {

    pub fn new(title: &str, icon: &str) -> Self {

        todo!()

    }

    pub fn set_attention_icon(&mut self, icon: &str) {

        todo!()

    }

    pub fn show(&mut self, attention: bool) {

        todo!()

    }

    pub fn hide(&mut self) {

        todo!()

    }

    pub fn add_label(&mut self, label: &str) {

        todo!()

    }

    pub fn add_menu_item<F>(&mut self, label: &str, cb: F)
        where F: Fn(()) -> () + 'static {

            todo!()

    }

}

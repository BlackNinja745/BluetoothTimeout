use eframe::egui::{Button, Response, Ui, Widget};
use crate::app::BluetoothApp;

pub struct RescanButtonSpinner<'a> {
    pub app: &'a mut BluetoothApp,
}

impl Widget for RescanButtonSpinner<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut response = ui.add_enabled(self.app.scan_recv.is_none(), Button::new("Rescan"));

        if response.clicked() {
            self.app.start_scan();
        }

        if self.app.scan_recv.is_some() {
            response |= ui.spinner();
        }

        response
    }
}
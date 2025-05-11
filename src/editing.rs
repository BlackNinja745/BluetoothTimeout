use eframe::egui::{Response, Ui, Widget};
use humantime::parse_duration;
use windows_bluetooth::{BluetoothDevice, MacAddress};
use crate::timeout::Timeout;

#[derive(Debug)]
pub struct TimeoutEditing {
    pub mac_address: MacAddress,
    pub text_edit_buffer: String,
    pub buffer_prev_had_focus: bool,
}

impl TimeoutEditing {
    pub fn new_of(mac_address: MacAddress) -> Self {
        Self {
            mac_address,
            text_edit_buffer: String::with_capacity(16),
            buffer_prev_had_focus: false,
        }
    }
}

pub struct TimeoutEditUi<'a> {
    pub device: &'a BluetoothDevice,
    pub timeout: &'a mut Timeout,
    pub editing: &'a mut TimeoutEditing,
}

impl Widget for TimeoutEditUi<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let name = self.device.name.as_deref().unwrap_or("Unknown");

            ui.label(format!("Editing {name}"));

            ui.checkbox(&mut self.timeout.remove_on_close, "Remove on close")
                .on_hover_text("Remove this device when the application closes");

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!("Timeout: {}", self.timeout.duration_str_or_none()));

                if self.timeout.duration.is_some() && ui.button("Remove").clicked() {
                    self.timeout.duration = None;
                }
            });


            let text_edit_resp = ui.horizontal(|ui| {
                ui.label("New Timeout: ");

                ui.text_edit_singleline(&mut self.editing.text_edit_buffer)
            })
                .inner;

            let edit_has_focus = text_edit_resp.has_focus();

            if self.editing.buffer_prev_had_focus && !edit_has_focus {
                if let Ok(duration) = parse_duration(&self.editing.text_edit_buffer) {
                    self.timeout.duration = Some(duration)
                }
            }

            if !edit_has_focus {
                self.editing.text_edit_buffer = self.timeout.duration_str().unwrap_or_default();

                self.editing.buffer_prev_had_focus = false;
            }

            if text_edit_resp.clicked() {
                self.editing.buffer_prev_had_focus = true;
            }
        }).response
    }
}
use std::fmt::{Display, Formatter};
use eframe::egui::{Align, Color32, FontId, Frame, Layout, Margin, Response, Ui, Widget};
use eframe::egui::text::LayoutJob;
use tokio::sync::mpsc::Sender;
use windows_bluetooth::BluetoothDevice;
use crate::app::{BluetoothApp, ConnectToDeviceRes, RemoveDeviceRes};
use crate::editing::TimeoutEditing;
use crate::timeout::Timeout;

pub struct ConnectedDeviceCard<'a> {
    pub outer_margin: Margin,
    pub remove_tx: Sender<RemoveDeviceRes>,
    pub device: &'a BluetoothDevice,
    pub timeout: &'a Timeout,
    pub editing: &'a mut Option<TimeoutEditing>,
}

impl Widget for ConnectedDeviceCard<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::new()
            .fill(Color32::from_gray(240))
            .outer_margin(self.outer_margin)
            .inner_margin(Margin::same(5))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let name = self.device.name.as_deref().unwrap_or("Unknown");

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui|
                            ui.label(TruncatedName(name, 20).to_string())
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui|
                            if ui.button("Remove").clicked() {
                                BluetoothApp::start_remove_with_tx(self.remove_tx, self.device.mac_address);
                            }
                        )
                    });

                    ui.horizontal(|ui| {
                        let left = ui.with_layout(Layout::left_to_right(Align::Center),|ui|
                            ui.label(format!("Timeout: {}", self.timeout.duration_str_or_none()))
                        );
                        
                        let right = ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Edit").clicked() {
                                *self.editing = Some(TimeoutEditing::new_of(self.timeout.mac_address))
                            }
                        });
                        
                        left.response | right.response
                    })
                })
            })
            .response
            .on_hover_text(LayoutJob::simple(
                format!(
                    "MAC Address: {}\nConnected: {}\nRemembered: {}\nLast Used: {:?}\nLast Seen: {:?}",
                    self.device.mac_address,
                    self.device.connected,
                    self.device.remembered,
                    self.device.last_used,
                    self.device.last_seen,
                ),
                FontId::proportional(11.0),
                ui.style().visuals.text_color(),
                5.0,
            ))
    }
}

pub struct AvailableDeviceCard<'a> {
    pub outer_margin: Margin,
    pub connect_tx: Sender<ConnectToDeviceRes>,
    pub device: &'a BluetoothDevice,
}

impl Widget for AvailableDeviceCard<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::new()
            .fill(Color32::from_gray(240))
            .outer_margin(self.outer_margin)
            .inner_margin(Margin::same(5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let name = self.device.name.as_deref().unwrap_or("Unknown");

                    ui.with_layout(Layout::left_to_right(Align::Center), |ui|
                        ui.label(TruncatedName(name, 20).to_string())
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui|
                        if ui.button("Connect").clicked() {
                            BluetoothApp::start_connect_with_tx(self.connect_tx, self.device.mac_address);
                        }
                    )
                })
            })
            .response
            .on_hover_text(LayoutJob::simple(
                format!(
                    "MAC Address: {}\nRemembered: {}\nLast Used: {:?}\nLast Seen: {:?}",
                    self.device.mac_address,
                    self.device.remembered,
                    self.device.last_used,
                    self.device.last_seen,
                ),
                FontId::proportional(11.0),
                ui.style().visuals.text_color(),
                5.0,
            ))
    }
}

struct TruncatedName<'a>(&'a str, usize);

impl Display for TruncatedName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.len() > self.1 {
            write!(f, "{}…", &self.0[..self.1 - 1])
        } else {
            write!(f, "{}", self.0)
        }
    }
}
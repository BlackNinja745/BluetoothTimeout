use pollster::FutureExt;
use std::time::{Duration, Instant};
use eframe::egui;
use eframe::egui::{Align, Color32, Frame, Layout, Margin, RichText, Vec2, Window};
use eframe::epaint::Stroke;
use eframe::glow::Context;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::sync::oneshot::{self, Receiver as OnceReceiver};
use tracing::debug;
use windows_bluetooth::{connect_to_device_os, discover_devices, remove_device, BluetoothDevice, ConnectToDeviceError, DiscoverDevicesError, MacAddress, RemoveDeviceError};
use crate::card::{AvailableDeviceCard, ConnectedDeviceCard};
use crate::editing::{TimeoutEditUi, TimeoutEditing};
use crate::spinner::RescanButtonSpinner;
use crate::timeout::Timeout;

pub type RemoveDeviceRes = Result<(), RemoveDeviceError>;
pub type ConnectToDeviceRes = Result<(), ConnectToDeviceError>;

pub struct BluetoothApp {
    pub devices: Vec<BluetoothDevice>,
    pub timeouts: Vec<Timeout>,
    pub connect_res_channel: (Sender<ConnectToDeviceRes>, Receiver<ConnectToDeviceRes>),
    pub remove_res_channel: (Sender<RemoveDeviceRes>, Receiver<RemoveDeviceRes>),
    pub scan_recv: Option<OnceReceiver<Result<Vec<BluetoothDevice>, DiscoverDevicesError>>>,
    pub last_timeout_update: Instant,
    pub editing: Option<TimeoutEditing>,
}

impl BluetoothApp {
    pub fn new_now() -> Self {
        Self {
            devices: Vec::default(),
            timeouts: Vec::default(),
            connect_res_channel: mpsc::channel(2),
            remove_res_channel: mpsc::channel(2),
            scan_recv: None,
            last_timeout_update: Instant::now(),
            editing: None,
        }
    }

    pub fn start_scan(&mut self) {
        debug!("Scanning for bluetooth devices");
        let (tx, rx) = oneshot::channel();

        task::spawn_blocking(move ||
            tx.send(discover_devices()).expect("should be able to send")
        );

        self.scan_recv = Some(rx);
    }

    pub fn start_connect_with_tx(tx: Sender<ConnectToDeviceRes>, mac_address: MacAddress) {
        task::spawn_blocking(move || tx.blocking_send(connect_to_device_os(mac_address)));
    }

    pub fn start_remove_with_tx(tx: Sender<RemoveDeviceRes>, mac_address: MacAddress) {
        task::spawn_blocking(move || tx.blocking_send(remove_device(mac_address)));
    }

    pub fn try_update_with_scan_result(&mut self) {
        if let Some(mut rx) = self.scan_recv.take() {
            if let Ok(devices) = rx.try_recv() {
                match devices {
                    Ok(mut devices) => {
                        devices.sort_by_key(|bd| bd.mac_address);
                        self.devices = devices;
                    }
                    Err(err) => tracing::error!("{err}"),
                }
                self.scan_recv = None;
            } else {
                self.scan_recv = Some(rx);
            }
        }
    }

    pub fn check_remove_connect_res(&mut self) {
        let mut updated = false;

        while let Ok(res) = self.remove_res_channel.1.try_recv() {
            updated = true;

            if let Err(err) = res {
                tracing::error!("{err}");
            }
        }

        while let Ok(res) = self.connect_res_channel.1.try_recv() {
            updated = true;

            if let Err(err) = res {
                tracing::error!("{err}");
            }
        }

        if updated {
            self.start_scan();
        }
    }

    pub fn process_timeout(&mut self) {
        self.timeouts.retain_mut(|timeout| {
            if let Some(ref mut duration) = timeout.duration {
                match duration.checked_sub(self.last_timeout_update.elapsed()) {
                    Some(res) => *duration = res,
                    None => {
                        *duration = Duration::ZERO;

                        let tx = self.remove_res_channel.0.clone();
                        let mac_address = timeout.mac_address;

                        Self::start_remove_with_tx(tx, mac_address);
                    }
                }
            }

            self.devices.iter().any(|bd| bd.connected && bd.mac_address == timeout.mac_address)
        });

        for bd in &self.devices {
            if bd.connected && !self.timeouts.iter().any(|timeout| bd.mac_address == timeout.mac_address) {
                self.timeouts.push(Timeout::default_from(bd.mac_address))
            }
        }

        self.last_timeout_update = Instant::now();
    }
}

impl eframe::App for BluetoothApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.try_update_with_scan_result();
        self.check_remove_connect_res();
        self.process_timeout();

        ctx.request_repaint_after_secs(2.0);

        if let Some(editing) = self.editing.as_mut() {
            let opt_device = self.devices
                .iter()
                .find(|d| d.mac_address == editing.mac_address);

            let opt_timeout = self.timeouts
                .iter_mut()
                .find(|d| d.mac_address == editing.mac_address);

            if let (Some(device), Some(timeout)) = (opt_device, opt_timeout) {
                let screen_size = ctx.screen_rect().size();
                let popup_size = Vec2::new(200.0, 200.0);

                let pos = (screen_size - popup_size) * 0.5;

                let mut keep_open = true;

                Window::new("Edit Timeout")
                    .resizable(false)
                    .movable(false)
                    .default_size(popup_size)
                    .collapsible(false)
                    .open(&mut keep_open)
                    .fixed_pos([pos.x, pos.y])
                    .show(ctx, |ui| ui.add(TimeoutEditUi { device, timeout, editing }));

                if !keep_open {
                    self.editing = None;
                }
            } else {
                self.editing = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let card_margin = Margin::same(2);

            let any_connected = self.devices.iter().any(|bd| bd.connected);

            ui.scope(|ui| {
                if self.editing.is_some() {
                    ui.disable();
                }

                if any_connected {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui|
                            ui.heading("Connected")
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui|
                            ui.add(RescanButtonSpinner { app: self })
                        )
                    });

                    egui::ScrollArea::vertical()
                        .id_salt("connected_scroll_area")
                        .max_height(200.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for device in self.devices.iter().filter(|&bd| bd.connected).cloned() {
                                let timeout = if let Some(timeout) = self.timeouts
                                    .iter_mut()
                                    .find(|t| t.mac_address == device.mac_address)
                                {
                                    timeout
                                } else {
                                    self.timeouts.push(Timeout::default_from(device.mac_address));

                                    self.timeouts.last_mut()
                                        .expect("should exist")
                                };

                                ui.add(ConnectedDeviceCard {
                                    outer_margin: card_margin,
                                    remove_tx: self.remove_res_channel.0.clone(),
                                    device: &device,
                                    timeout,
                                    editing: &mut self.editing,
                                });
                            }
                        });

                    ui.separator();
                }

                if self.devices.iter().any(|bd| !bd.connected) {
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui|
                            ui.heading("Available")
                        );

                        if !any_connected {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(RescanButtonSpinner { app: self })
                            });
                        }
                    });

                    egui::ScrollArea::vertical()
                        .id_salt("available_scroll_area")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for device in self.devices.iter()
                                .filter(|bd| !bd.connected)

                            {
                                ui.add(AvailableDeviceCard {
                                    connect_tx: self.connect_res_channel.0.clone(),
                                    outer_margin: card_margin,
                                    device,
                                });
                            }
                        });
                }

                if self.devices.is_empty() {
                    Frame::new()
                        .corner_radius(8.0)
                        .stroke(Stroke::new(2.0, Color32::DARK_GRAY))
                        .inner_margin(Margin::same(4))
                        .fill(Color32::LIGHT_GRAY)
                        .show(ui, |ui| ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No Devices Found").heading());
                            ui.add(RescanButtonSpinner { app: self });
                        }));
                }
            })
        });
    }

    fn on_exit(&mut self, _: Option<&Context>) {
        let mut handles = Vec::new();

        for timeout in self.timeouts.drain(..) {
            if timeout.remove_on_close {
                handles.push(
                    task::spawn_blocking(move || remove_device(timeout.mac_address))
                );
            }
        }

        for handle in handles {
            if let Ok(Err(err)) = handle.into_future().block_on() {
                tracing::error!("{err}");
            }
        }
    }
}
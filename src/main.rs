#![windows_subsystem = "windows"]

mod app;
mod editing;
mod card;
mod spinner;
mod timeout;

use std::str::FromStr;
use eframe::egui::ViewportBuilder;
use tracing_subscriber::EnvFilter;
use app::BluetoothApp;

#[tokio::main]
async fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_str("warn,bluetooth_timeout=debug")
                .expect("failed to set default env filter")
        )
        .init();

    eframe::run_native(
        "Bluetooth App",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_resizable(false)
                .with_maximize_button(false)
                .with_inner_size((250.0, 275.0)),
                // .with_icon()
                .. Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.style_mut(|style| style.visuals.dark_mode = true);

            Ok(Box::new(BluetoothApp::new_now()))
        }),
    )
}
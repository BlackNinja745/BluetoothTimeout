#![windows_subsystem = "windows"]

mod app;
mod editing;
mod card;
mod spinner;
mod timeout;
mod logging;
mod notification;

use eframe::egui::ViewportBuilder;
use eframe::icon_data;
use tokio::sync::mpsc;
use app::BluetoothApp;

#[tokio::main(flavor="current_thread")]
async fn main() -> eframe::Result {
    // for logging
    let (tx, rx) = mpsc::channel(5);
    
    logging::init(tx).expect("init shouldn't fail");
    
    let icon = icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).expect("png bytes should be valid");
    
    eframe::run_native(
        "Bluetooth Timeout",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_resizable(false)
                .with_maximize_button(false)
                .with_inner_size((250.0, 275.0))
                .with_icon(icon),
                .. Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.style_mut(|style| style.visuals.dark_mode = true);

            Ok(Box::new(BluetoothApp::new_now_with_log_rx(rx)))
        }),
    )
}
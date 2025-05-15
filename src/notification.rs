use eframe::egui::{Color32, Frame, Margin, Response, RichText, Stroke, Ui, Widget};
use tracing::Level;
use crate::logging::Log;

pub struct Notification<'a>(pub &'a Log);

impl Widget for Notification<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let color = match self.0.level {
            Level::DEBUG => Color32::LIGHT_BLUE,
            Level::WARN => Color32::ORANGE,
            Level::ERROR => Color32::RED,
            Level::INFO => Color32::GREEN,
            Level::TRACE => Color32::LIGHT_GRAY,
        };
        
        Frame::popup(ui.style())
            .stroke(Stroke::new(1.0, color))
            .inner_margin(Margin::same(8))
            .show(ui, |ui| ui.vertical(|ui| {
                ui.label(RichText::new(format!("{:<8}", self.0.level)).color(color).strong())
                | ui.label(RichText::new(&self.0.message).size(10.0))
            }))
            .response
    }
}
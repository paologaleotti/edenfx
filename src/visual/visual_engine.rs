use crate::controller::ControllerOutput;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct VisualEngine {
    controller_output: Arc<Mutex<ControllerOutput>>,
}

impl VisualEngine {
    pub fn new(controller_output: Arc<Mutex<ControllerOutput>>) -> Self {
        Self { controller_output }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        // TODO: use controller_output to drive visuals

        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::BLACK)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.colored_label(
                        egui::Color32::WHITE,
                        egui::RichText::new("EDENfx").heading(),
                    );
                });
            });

        ctx.request_repaint();
    }
}

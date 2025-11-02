mod analyzer;
mod audio_stream;
mod config;
mod gui;

use eframe;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_title("EDEN audio visualizer"),
        ..Default::default()
    };

    eframe::run_native(
        "EDEN audio visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(gui::AppState::default()))),
    )
}

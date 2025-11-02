mod analyzer;
mod audio_stream;
mod config;
mod controller;
mod gui;

use analyzer::{AudioAnalyzer, AudioMetrics};
use config::AudioConfig;
use controller::{Controller, ControllerOutput};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), eframe::Error> {
    // === Shared State ===
    let analyzer_metrics = Arc::new(Mutex::new(AudioMetrics::default()));
    let controller_output = Arc::new(Mutex::new(ControllerOutput::default()));
    let config = Arc::new(Mutex::new(AudioConfig::default()));

    // === Analyzer Setup ===
    let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new(config.clone())));

    // === Analysis Thread ===
    let analyzer_thread = {
        let analyzer = analyzer.clone();
        let metrics = analyzer_metrics.clone();
        let config = config.clone();

        thread::spawn(move || {
            loop {
                let interval = config.lock().unwrap().update_interval_ms;
                thread::sleep(Duration::from_millis(interval));

                let new_metrics = analyzer.lock().unwrap().analyze();
                *metrics.lock().unwrap() = new_metrics;
            }
        })
    };

    // === Controller Thread ===
    let controller_thread = {
        let controller = Controller::new(config.clone());
        let metrics = analyzer_metrics.clone();
        let output = controller_output.clone();
        let config = config.clone();

        thread::spawn(move || {
            loop {
                let interval = config.lock().unwrap().update_interval_ms;
                thread::sleep(Duration::from_millis(interval));

                let current_metrics = metrics.lock().unwrap().clone();
                let new_output =
                    controller.process(current_metrics.loudness, current_metrics.bass_energy);
                *output.lock().unwrap() = new_output;
            }
        })
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_title("EDEN audio visualizer"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "EDEN audio visualizer",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(gui::AppState::new(
                config,
                analyzer,
                analyzer_metrics,
                controller_output,
            )))
        }),
    );

    // Note: threads will be terminated when the program exits
    drop(analyzer_thread);
    drop(controller_thread);

    result
}

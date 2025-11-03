mod audio;
mod config;
mod controller;
mod gui;
mod visual;

use audio::{AudioAnalyzer, AudioMetrics};
use controller::{Controller, ControllerOutput};
use log::{debug, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::config::AudioConfig;

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("Starting up...");

    // === Shared State ===
    let analyzer_metrics = Arc::new(Mutex::new(AudioMetrics::default()));
    let controller_output = Arc::new(Mutex::new(ControllerOutput::default()));
    let config = Arc::new(Mutex::new(AudioConfig::default()));
    let shutdown = Arc::new(AtomicBool::new(false));

    // === Analyzer Setup ===
    let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new(config.clone())));

    // === Analysis Thread ===
    debug!("Spawning analyzer thread...");
    let analyzer_thread = {
        let analyzer = analyzer.clone();
        let metrics = analyzer_metrics.clone();
        let config = config.clone();
        let shutdown = shutdown.clone();

        thread::spawn(move || {
            debug!("Analyzer thread started");
            while !shutdown.load(Ordering::Relaxed) {
                let interval = config.lock().unwrap().update_interval_ms;
                thread::sleep(Duration::from_millis(interval));

                if !shutdown.load(Ordering::Relaxed) {
                    let new_metrics = analyzer.lock().unwrap().analyze();
                    *metrics.lock().unwrap() = new_metrics;
                }
            }
            debug!("Analyzer thread shutting down");
        })
    };

    // === Controller Thread ===
    debug!("Spawning controller thread...");
    let controller_thread = {
        let controller = Controller::new(config.clone());
        let metrics = analyzer_metrics.clone();
        let output = controller_output.clone();
        let config = config.clone();
        let shutdown = shutdown.clone();

        thread::spawn(move || {
            debug!("Controller thread started");
            while !shutdown.load(Ordering::Relaxed) {
                let interval = config.lock().unwrap().update_interval_ms;
                thread::sleep(Duration::from_millis(interval));

                if !shutdown.load(Ordering::Relaxed) {
                    let current_metrics = metrics.lock().unwrap().clone();
                    let new_output = controller.process(current_metrics);
                    *output.lock().unwrap() = new_output;
                }
            }
            debug!("Controller thread shutting down");
        })
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 700.0])
            .with_title("EDEN Controller"),
        ..Default::default()
    };

    debug!("Launching GUI...");
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

    debug!("Signaling threads to shut down...");
    shutdown.store(true, Ordering::Relaxed);

    debug!("Waiting for analyzer thread to finish...");
    analyzer_thread
        .join()
        .expect("Failed to join analyzer thread");
    debug!("Analyzer thread joined");

    debug!("Waiting for controller thread to finish...");
    controller_thread
        .join()
        .expect("Failed to join controller thread");
    debug!("Controller thread joined");

    info!("Clean shutdown complete");

    result
}

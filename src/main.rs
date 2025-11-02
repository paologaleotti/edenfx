mod analyzer;
mod audio_stream;
mod config;
mod controller;
mod gui;

use analyzer::{AudioAnalyzer, AudioMetrics};
use audio_stream::AudioStream;
use config::AudioConfig;
use controller::{Controller, ControllerOutput};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), eframe::Error> {
    // === Shared State ===
    let is_running = Arc::new(Mutex::new(false));
    let analyzer_metrics = Arc::new(Mutex::new(AudioMetrics::default()));
    let controller_output = Arc::new(Mutex::new(ControllerOutput::default()));

    let config = AudioConfig::default();
    let update_interval_ms = config.update_interval_ms;

    // === Audio Stream Setup ===
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("No input device available");
    let supported_config = device
        .default_input_config()
        .expect("Failed to get default input config");
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();

    let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new(config.clone())));

    let _audio_stream = AudioStream::new(&device, &stream_config, sample_format, analyzer.clone())
        .expect("Failed to create audio stream");

    let analyzer_thread = {
        let analyzer = analyzer.clone();
        let metrics = analyzer_metrics.clone();
        let is_running = is_running.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(update_interval_ms));

                if *is_running.lock().unwrap() {
                    let new_metrics = analyzer.lock().unwrap().analyze();
                    *metrics.lock().unwrap() = new_metrics;
                }
            }
        })
    };

    let controller_thread = {
        let controller = Controller::new(config.clone());
        let metrics = analyzer_metrics.clone();
        let output = controller_output.clone();
        let is_running = is_running.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(update_interval_ms));

                if *is_running.lock().unwrap() {
                    let current_metrics = metrics.lock().unwrap().clone();
                    let new_output =
                        controller.process(current_metrics.loudness, current_metrics.bass_energy);
                    *output.lock().unwrap() = new_output;
                }
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
                is_running,
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

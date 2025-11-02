use crate::analyzer::{AudioAnalyzer, AudioMetrics};
use crate::audio_stream::{self, AudioStream};
use crate::config::APP_VERSION;
use crate::config::AudioConfig;
use crate::controller::ControllerOutput;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;
use log::{debug, info};
use std::sync::{Arc, Mutex};

pub struct AppState {
    active_config: Arc<Mutex<AudioConfig>>,
    pending_config: AudioConfig, // Local copy for sliders
    devices: Vec<String>,
    active_device_idx: usize,
    pending_device_idx: usize, // Local selection for device selector
    analyzer: Arc<Mutex<AudioAnalyzer>>,
    audio_stream: Option<AudioStream>,
    analyzer_metrics: Arc<Mutex<AudioMetrics>>,
    controller_output: Arc<Mutex<ControllerOutput>>,
    visualizer_open: bool,
}

impl AppState {
    pub fn new(
        config: Arc<Mutex<AudioConfig>>,
        analyzer: Arc<Mutex<AudioAnalyzer>>,
        analyzer_metrics: Arc<Mutex<AudioMetrics>>,
        controller_output: Arc<Mutex<ControllerOutput>>,
    ) -> Self {
        debug!("Initializing GUI state...");
        let host = cpal::default_host();

        let devices: Vec<String> = host
            .input_devices()
            .ok()
            .map(|iter| iter.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default();

        debug!("Found {} audio input devices", devices.len());

        let default_device_name = host.default_input_device().and_then(|d| d.name().ok());

        let selected_device_idx = if let Some(ref default_name) = default_device_name {
            devices
                .iter()
                .position(|name| name == default_name)
                .unwrap_or(0)
        } else {
            0
        };

        let selected_device = devices
            .get(selected_device_idx)
            .map(|s| s.as_str())
            .unwrap_or("None");
        info!("Selected initial audio device: {selected_device}");

        let audio_stream =
            audio_stream::create_audio_stream(selected_device_idx, &devices, analyzer.clone());

        let pending_config = config.lock().unwrap().clone();
        debug!(
            "Initial config loaded: sample_rate={}, buffer_size={}, update_interval={}ms",
            pending_config.sample_rate,
            pending_config.buffer_size,
            pending_config.update_interval_ms
        );

        Self {
            active_config: config,
            pending_config,
            devices,
            pending_device_idx: selected_device_idx,
            active_device_idx: selected_device_idx,
            analyzer,
            audio_stream,
            analyzer_metrics,
            controller_output,
            visualizer_open: false,
        }
    }

    fn apply_settings(&mut self) {
        let device_name = self
            .devices
            .get(self.pending_device_idx)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");

        debug!(
            "Applying settings - Device: {}, Config: {:?}",
            device_name, self.pending_config
        );

        // Lock and copy pending config to shared config
        {
            let mut config = self.active_config.lock().unwrap();
            *config = self.pending_config.clone();
        }

        debug!("Reloading audio stream with new device...");
        self.audio_stream = audio_stream::create_audio_stream(
            self.pending_device_idx,
            &self.devices,
            self.analyzer.clone(),
        );

        self.active_device_idx = self.pending_device_idx;
        info!("Settings applied successfully");
    }

    fn reset_to_default(&mut self) {
        debug!("Resetting config to defaults");
        let default_config = AudioConfig::default();
        self.pending_config = default_config.clone();
    }

    fn disable_apply_button(&self) -> bool {
        let config_unchanged = self.pending_config == *self.active_config.lock().unwrap();
        let device_unchanged = self.pending_device_idx == self.active_device_idx;

        config_unchanged && device_unchanged
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("EDEN {APP_VERSION}"));
            ui.separator();

            // Device Selection and Settings
            ui.group(|ui| {
                ui.label("Audio Input Device:");
                egui::ComboBox::from_id_salt("device_selector")
                    .selected_text(
                        self.devices
                            .get(self.pending_device_idx)
                            .map(|name| name.as_str())
                            .unwrap_or("No devices"),
                    )
                    .show_ui(ui, |ui| {
                        for (idx, name) in self.devices.iter().enumerate() {
                            ui.selectable_value(&mut self.pending_device_idx, idx, name);
                        }
                    });

                if self.audio_stream.is_some() {
                    ui.colored_label(egui::Color32::GREEN, "Listening");
                } else {
                    ui.colored_label(egui::Color32::RED, "No audio stream");
                }
            });

            ui.separator();

            // Real-time Metrics Display
            let analyzer_metrics = self.analyzer_metrics.lock().unwrap().clone();
            let controller_output = self.controller_output.lock().unwrap().clone();

            // Analyzer Output (Debug)
            ui.group(|ui| {
                ui.colored_label(egui::Color32::LIGHT_BLUE, "Analyzer Output (Raw Metrics):");
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Loudness: {:.1}%",
                        analyzer_metrics.loudness * 100.0
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Bass Energy: {:.1}%",
                        analyzer_metrics.bass_energy * 100.0
                    ));
                });
            });

            ui.add_space(5.0);

            // Controller Output (Debug)
            ui.group(|ui| {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Controller Output:");
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Loudness (passthrough): {:.1}%",
                        controller_output.loudness * 100.0
                    ));
                });
                ui.horizontal(|ui| {
                    if controller_output.is_drop {
                        ui.colored_label(egui::Color32::RED, "DROP DETECTED");
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "Normal");
                    }
                });
            });
            ui.separator();

            // Configuration Sliders (modify pending config only)
            egui::CollapsingHeader::new("Bass Detection Settings")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("bass_settings_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Bass Freq Max (Hz):")
                                .on_hover_text("What counts as 'bass' - lower = only deep bass");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.pending_config.bass_freq_max,
                                    20.0..=500.0,
                                )
                                .suffix(" Hz"),
                            );
                            ui.end_row();

                            ui.label("Bass Sensitivity:")
                                .on_hover_text("Higher = more sensitive to bass");
                            ui.add(egui::Slider::new(
                                &mut self.pending_config.bass_energy_multiplier,
                                1.0..=5.0,
                            ));
                            ui.end_row();

                            ui.label("Drop Threshold:")
                                .on_hover_text("When to trigger DROP detection");
                            ui.add(egui::Slider::new(
                                &mut self.pending_config.drop_detection_threshold,
                                0.0..=1.0,
                            ));
                            ui.end_row();
                        });
                });

            egui::CollapsingHeader::new("General Settings")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("general_settings_grid")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Loudness Sensitivity:")
                                .on_hover_text("Higher = more sensitive to quiet sounds");
                            ui.add(egui::Slider::new(
                                &mut self.pending_config.loudness_multiplier,
                                5.0..=20.0,
                            ));
                            ui.end_row();

                            ui.label("Update Interval (ms):")
                                .on_hover_text("How often to analyze (lower = smoother)");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.pending_config.update_interval_ms,
                                    50..=500,
                                )
                                .suffix(" ms"),
                            );
                            ui.end_row();
                        });
                });

            ui.separator();

            ui.horizontal(|ui| {
                ui.add_enabled_ui(!self.disable_apply_button(), |ui| {
                    if ui.button("Apply and reload").clicked() {
                        self.apply_settings();
                    }
                });

                if ui.button("↺ Reset to default").clicked() {
                    self.reset_to_default();
                }
            });

            ui.separator();

            // Visualizer Window Button
            ui.horizontal(|ui| {
                if ui.button("Open Visualizer window").clicked() {
                    self.visualizer_open = true;
                }
                if self.visualizer_open {
                    ui.colored_label(egui::Color32::GREEN, "Visualizer Ready");
                }
            });
        });

        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

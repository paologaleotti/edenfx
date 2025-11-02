use crate::analyzer::AudioMetrics;
use crate::audio_stream::AudioStream;
use crate::config::APP_VERSION;
use crate::config::AudioConfig;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;

pub struct AppState {
    config: AudioConfig,
    devices: Vec<(usize, String)>,
    selected_device_idx: usize,
    audio_stream: Option<AudioStream>,
    metrics: AudioMetrics,
    visualizer_open: bool,
}

impl Default for AppState {
    fn default() -> Self {
        let host = cpal::default_host();
        let devices: Vec<(usize, String)> = host
            .input_devices()
            .ok()
            .map(|iter| {
                iter.enumerate()
                    .filter_map(|(i, d)| d.name().ok().map(|name| (i, name)))
                    .collect()
            })
            .unwrap_or_default();

        Self {
            config: AudioConfig::default(),
            devices,
            selected_device_idx: 0,
            audio_stream: None,
            metrics: AudioMetrics::default(),
            visualizer_open: false,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update metrics if stream is active
        if let Some(stream) = &self.audio_stream {
            self.metrics = stream.get_metrics();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("EDEN {}", APP_VERSION));
            ui.separator();

            // Device Selection
            ui.group(|ui| {
                ui.label("Audio Input Device:");
                egui::ComboBox::from_id_salt("device_selector")
                    .selected_text(
                        self.devices
                            .get(self.selected_device_idx)
                            .map(|(_, name)| name.as_str())
                            .unwrap_or("No devices"),
                    )
                    .show_ui(ui, |ui| {
                        for (idx, name) in &self.devices {
                            ui.selectable_value(&mut self.selected_device_idx, *idx, name);
                        }
                    });

                ui.horizontal(|ui| {
                    if self.audio_stream.is_some() {
                        if ui.button("⏹ Stop").clicked() {
                            self.audio_stream = None;
                        }
                    } else {
                        if ui.button("▶ Start Listening").clicked() {
                            self.start_audio();
                        }
                    }
                });
            });

            ui.separator();

            // Real-time Metrics Display
            if self.audio_stream.is_some() {
                ui.group(|ui| {
                    ui.label("Live Audio Metrics:");
                    ui.horizontal(|ui| {
                        ui.label(format!("Loudness: {:.1}%", self.metrics.loudness * 100.0));
                        ui.separator();
                        ui.label(format!("Bass: {:.1}%", self.metrics.bass_energy * 100.0));
                    });

                    ui.horizontal(|ui| {
                        if self.metrics.is_drop {
                            ui.colored_label(egui::Color32::RED, "DROP DETECTED");
                        } else {
                            ui.label("Normal");
                        }
                    });
                });
                ui.separator();
            }

            // Configuration Sliders
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
                                egui::Slider::new(&mut self.config.bass_freq_max, 20.0..=500.0)
                                    .suffix(" Hz"),
                            );
                            ui.end_row();

                            ui.label("Bass Sensitivity:")
                                .on_hover_text("Higher = more sensitive to bass");
                            ui.add(egui::Slider::new(
                                &mut self.config.bass_energy_multiplier,
                                1.0..=5.0,
                            ));
                            ui.end_row();

                            ui.label("Drop Threshold:")
                                .on_hover_text("When to trigger DROP detection");
                            ui.add(egui::Slider::new(
                                &mut self.config.drop_detection_threshold,
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
                                &mut self.config.loudness_multiplier,
                                5.0..=20.0,
                            ));
                            ui.end_row();

                            ui.label("Update Interval (ms):")
                                .on_hover_text("How often to analyze (lower = smoother)");
                            ui.add(
                                egui::Slider::new(&mut self.config.update_interval_ms, 50..=500)
                                    .suffix(" ms"),
                            );
                            ui.end_row();
                        });
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

impl AppState {
    fn start_audio(&mut self) {
        let host = cpal::default_host();

        let mut devices = match host.input_devices() {
            Ok(devices) => devices,
            Err(_) => {
                eprintln!("Failed to get input devices");
                return;
            }
        };

        let device = match devices.nth(self.selected_device_idx) {
            Some(device) => device,
            None => {
                eprintln!("Selected device not found");
                return;
            }
        };

        let supported_config = match device.default_input_config() {
            Ok(config) => config,
            Err(_) => {
                eprintln!("Failed to get default input config");
                return;
            }
        };

        let sample_format = supported_config.sample_format();
        let config: cpal::StreamConfig = supported_config.into();

        match AudioStream::new(&device, &config, sample_format, self.config.clone()) {
            Ok(stream) => {
                self.audio_stream = Some(stream);
                println!("Audio stream started");
            }
            Err(e) => eprintln!("Failed to start audio: {}", e),
        }
    }
}

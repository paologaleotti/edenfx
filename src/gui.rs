use crate::analyzer::AudioMetrics;
use crate::config::APP_VERSION;
use crate::config::AudioConfig;
use crate::controller::ControllerOutput;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct AppState {
    config: AudioConfig,
    devices: Vec<(usize, String)>,
    selected_device_idx: usize,
    is_running: Arc<Mutex<bool>>,
    analyzer_metrics: Arc<Mutex<AudioMetrics>>,
    controller_output: Arc<Mutex<ControllerOutput>>,
    visualizer_open: bool,
}

impl AppState {
    pub fn new(
        is_running: Arc<Mutex<bool>>,
        analyzer_metrics: Arc<Mutex<AudioMetrics>>,
        controller_output: Arc<Mutex<ControllerOutput>>,
    ) -> Self {
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
            is_running,
            analyzer_metrics,
            controller_output,
            visualizer_open: false,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_running = *self.is_running.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("EDEN {APP_VERSION}"));
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
                    if is_running {
                        if ui.button("⏹ Stop").clicked() {
                            *self.is_running.lock().unwrap() = false;
                        }
                    } else if ui.button("▶ Start Listening").clicked() {
                        *self.is_running.lock().unwrap() = true;
                    }
                });
            });

            ui.separator();

            // Real-time Metrics Display
            if is_running {
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

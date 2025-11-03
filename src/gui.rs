use crate::analyzer::{AudioAnalyzer, AudioMetrics};
use crate::audio_stream::{self, AudioStream};
use crate::config::APP_VERSION;
use crate::config::AudioConfig;
use crate::controller::ControllerOutput;
use crate::visual_engine::VisualEngine;
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
    visuals_window_open: bool,
    visuals_window: VisualEngine,
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

        let visuals_window = VisualEngine::new(controller_output.clone());

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
            visuals_window_open: false,
            visuals_window,
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
        // Top panel for title and device selection
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(format!("EDEN {APP_VERSION}"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.audio_stream.is_some() {
                        ui.colored_label(egui::Color32::GREEN, "Listening");
                    } else {
                        ui.colored_label(egui::Color32::RED, "No Audio Stream");
                    }
                });
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Device Selection
            ui.horizontal(|ui| {
                ui.label("Audio Device:");
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
            });
            ui.add_space(4.0);
        });

        // Bottom panel for action buttons
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let apply_enabled = !self.disable_apply_button();

                if apply_enabled {
                    if ui.button("Apply Settings").clicked() {
                        self.apply_settings();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("Apply Settings"));
                }

                if ui.button("↺ Reset to Default").clicked() {
                    self.reset_to_default();
                }

                ui.separator();

                ui.add_enabled_ui(!self.visuals_window_open, |ui| {
                    if ui.button("Open Visualizer").clicked() {
                        self.visuals_window_open = true;
                        info!("Visualizer window opened");
                    }
                });

                if self.visuals_window_open {
                    ui.colored_label(egui::Color32::GREEN, "● Visuals Active");
                }
            });
            ui.add_space(8.0);
        });

        // Central scrollable panel
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(8.0);

                    // === LIVE MONITORING ===
                    ui.label(egui::RichText::new("Live Monitoring").size(16.0));
                    ui.add_space(8.0);

                    let analyzer_metrics = self.analyzer_metrics.lock().unwrap().clone();
                    let controller_output = self.controller_output.lock().unwrap().clone();

                    // Analyzer Output (Raw Metrics)
                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::LIGHT_BLUE,
                            "Analyzer Output (Raw Metrics):",
                        );
                        ui.horizontal(|ui| {
                            ui.label("Loudness:");
                            ui.strong(format!("{:.1}%", analyzer_metrics.loudness * 100.0));

                            ui.separator();

                            ui.label("Bass Energy:");
                            ui.strong(format!("{:.1}%", analyzer_metrics.bass_energy * 100.0));
                        });
                    });

                    ui.add_space(8.0);

                    // Controller Output
                    ui.group(|ui| {
                        ui.colored_label(egui::Color32::LIGHT_GREEN, "Controller Output:");
                        ui.horizontal(|ui| {
                            ui.label("Loudness (passthrough):");
                            ui.strong(format!("{:.1}%", controller_output.loudness * 100.0));

                            ui.separator();

                            if controller_output.is_drop {
                                ui.colored_label(egui::Color32::RED, "● DROP DETECTED");
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "○ Normal");
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // Waveform visualization
                    ui.group(|ui| {
                        ui.label("Waveform");

                        // Get the current audio buffer
                        let audio_buffer = self.analyzer.lock().unwrap().get_buffer();

                        // Create a custom waveform visualization
                        let desired_height = 120.0;
                        let (response, painter) = ui.allocate_painter(
                            egui::vec2(ui.available_width(), desired_height),
                            egui::Sense::hover(),
                        );

                        let rect = response.rect;

                        // Draw background
                        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));

                        // Draw center line
                        let center_y = rect.center().y;
                        painter.line_segment(
                            [
                                egui::pos2(rect.left(), center_y),
                                egui::pos2(rect.right(), center_y),
                            ],
                            egui::Stroke::new(0.5, egui::Color32::from_gray(80)),
                        );

                        if !audio_buffer.is_empty() {
                            let width = rect.width();
                            let height = rect.height();
                            let num_samples = audio_buffer.len();

                            // Build line segments for the waveform
                            let mut points: Vec<egui::Pos2> = Vec::with_capacity(num_samples);

                            for (i, &sample) in audio_buffer.iter().enumerate() {
                                let x = rect.left() + (i as f32 / num_samples as f32) * width;
                                // Map sample from [-1, 1] to screen coordinates
                                let y = center_y - (sample * height * 0.45);
                                points.push(egui::pos2(x, y));
                            }

                            // Draw the waveform
                            if points.len() > 1 {
                                painter.add(egui::Shape::line(
                                    points,
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 200, 255)),
                                ));
                            }
                        }
                    });

                    ui.add_space(20.0);

                    // === CONFIGURATION ===
                    ui.label(egui::RichText::new("Configuration").size(16.0));
                    ui.add_space(8.0);

                    // Bass Detection Settings
                    egui::CollapsingHeader::new("Bass Detection")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            egui::Grid::new("bass_settings_grid")
                                .num_columns(2)
                                .spacing([20.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("Max Bass Freq:").on_hover_text(
                                        "What counts as 'bass' - lower = only deep bass",
                                    );
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

                    ui.add_space(8.0);

                    // Audio Processing Settings
                    egui::CollapsingHeader::new("Audio Processing")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            egui::Grid::new("general_settings_grid")
                                .num_columns(2)
                                .spacing([20.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("Loudness Sensitivity:")
                                        .on_hover_text("Higher = more sensitive to quiet sounds");
                                    ui.add(egui::Slider::new(
                                        &mut self.pending_config.loudness_multiplier,
                                        5.0..=20.0,
                                    ));
                                    ui.end_row();

                                    ui.label("Update Interval:")
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

                    ui.add_space(20.0);
                });
        });

        // Visualizer Window (Separate OS Window)
        if self.visuals_window_open {
            let visualizer_id = egui::ViewportId::from_hash_of("edenfx_visualizer");

            ctx.show_viewport_immediate(
                visualizer_id,
                egui::ViewportBuilder::default()
                    .with_title("EDEN Visuals")
                    .with_inner_size([800.0, 600.0])
                    .with_resizable(true),
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.visuals_window_open = false;
                    }
                    self.visuals_window.render(ctx);
                },
            );
        }

        ctx.request_repaint();
    }
}

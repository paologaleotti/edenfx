use crate::audio::{AudioAnalyzer, AudioMetrics, AudioStream, audio_stream};
use crate::config::{APP_VERSION, AudioConfig};
use crate::controller::ControllerOutput;
use crate::visual::VisualEngine;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui;
use log::{debug, info};
use std::sync::{Arc, Mutex};

use super::components::{render_config_panel, render_live_monitoring, render_waveform};

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

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_top_panel(ctx);
        self.render_bottom_panel(ctx);
        self.render_central_panel(ctx);
        self.render_visualizer_window(ctx);

        ctx.request_repaint();
    }
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

    fn render_top_panel(&mut self, ctx: &egui::Context) {
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
    }

    fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
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
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.add_space(8.0);

                    // Live Monitoring Section
                    let analyzer_metrics = self.analyzer_metrics.lock().unwrap().clone();
                    let controller_output = self.controller_output.lock().unwrap().clone();
                    render_live_monitoring(ui, &analyzer_metrics, &controller_output);

                    // Waveform Visualization
                    let audio_buffer = self.analyzer.lock().unwrap().get_buffer();
                    render_waveform(ui, &audio_buffer);

                    ui.add_space(20.0);

                    // Configuration Section
                    render_config_panel(ui, &mut self.pending_config);
                });
        });
    }

    fn render_visualizer_window(&mut self, ctx: &egui::Context) {
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
    }
}

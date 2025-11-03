use eframe::egui;

use crate::config::AudioConfig;

pub fn render_config_panel(ui: &mut egui::Ui, config: &mut AudioConfig) {
    ui.label(egui::RichText::new("Configuration").size(16.0));
    ui.add_space(8.0);

    // Bass Detection Settings
    render_bass_detection(ui, config);

    ui.add_space(8.0);

    // Audio Processing Settings
    render_audio_processing(ui, config);

    ui.add_space(20.0);
}

fn render_bass_detection(ui: &mut egui::Ui, config: &mut AudioConfig) {
    egui::CollapsingHeader::new("Bass Detection")
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(4.0);
            egui::Grid::new("bass_settings_grid")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Max Bass Freq:")
                        .on_hover_text("What counts as 'bass' - lower = only deep bass");
                    ui.add(
                        egui::Slider::new(&mut config.bass_freq_max, 20.0..=500.0).suffix(" Hz"),
                    );
                    ui.end_row();

                    ui.label("Bass Sensitivity:")
                        .on_hover_text("Higher = more sensitive to bass");
                    ui.add(egui::Slider::new(
                        &mut config.bass_energy_multiplier,
                        1.0..=5.0,
                    ));
                    ui.end_row();

                    ui.label("Drop Threshold:")
                        .on_hover_text("When to trigger DROP detection");
                    ui.add(egui::Slider::new(
                        &mut config.drop_detection_threshold,
                        0.0..=1.0,
                    ));
                    ui.end_row();
                });
        });
}

fn render_audio_processing(ui: &mut egui::Ui, config: &mut AudioConfig) {
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
                        &mut config.loudness_multiplier,
                        5.0..=20.0,
                    ));
                    ui.end_row();

                    ui.label("Update Interval:")
                        .on_hover_text("How often to analyze (lower = smoother)");
                    ui.add(
                        egui::Slider::new(&mut config.update_interval_ms, 50..=500).suffix(" ms"),
                    );
                    ui.end_row();
                });
        });
}

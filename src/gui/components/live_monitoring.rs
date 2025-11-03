use crate::audio::AudioMetrics;
use crate::controller::ControllerOutput;
use eframe::egui;

pub fn render_live_monitoring(
    ui: &mut egui::Ui,
    analyzer_metrics: &AudioMetrics,
    controller_output: &ControllerOutput,
) {
    ui.label(egui::RichText::new("Live Monitoring").size(16.0));
    ui.add_space(8.0);

    render_analyzer_metrics(ui, analyzer_metrics);
    ui.add_space(8.0);

    render_controller_output(ui, controller_output);
    ui.add_space(12.0);
}

fn render_analyzer_metrics(ui: &mut egui::Ui, metrics: &AudioMetrics) {
    ui.group(|ui| {
        ui.colored_label(egui::Color32::LIGHT_BLUE, "Analyzer Output (Raw Metrics):");
        ui.horizontal(|ui| {
            ui.label("Loudness:");
            ui.strong(format!("{:.1}%", metrics.loudness * 100.0));

            ui.separator();

            ui.label("Bass Energy:");
            ui.strong(format!("{:.1}%", metrics.bass_energy * 100.0));
        });
    });
}

fn render_controller_output(ui: &mut egui::Ui, output: &ControllerOutput) {
    ui.group(|ui| {
        ui.colored_label(egui::Color32::LIGHT_GREEN, "Controller Output:");
        ui.horizontal(|ui| {
            ui.label("Loudness (passthrough):");
            ui.strong(format!("{:.1}%", output.loudness * 100.0));

            ui.separator();

            if output.is_drop {
                ui.colored_label(egui::Color32::RED, "DROP DETECTED");
            } else {
                ui.colored_label(egui::Color32::GRAY, "Normal");
            }
        });
    });
}

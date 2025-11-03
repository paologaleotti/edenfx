use eframe::egui;

pub fn render_waveform(ui: &mut egui::Ui, audio_buffer: &[f32]) {
    ui.group(|ui| {
        ui.label("Waveform");

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
}

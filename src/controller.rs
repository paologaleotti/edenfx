use crate::config::AudioConfig;

#[derive(Clone, Default, Debug)]
pub struct ControllerOutput {
    pub is_drop: bool,
    pub loudness: f32,
}

pub struct Controller {
    config: AudioConfig,
}

impl Controller {
    pub fn new(config: AudioConfig) -> Self {
        Self { config }
    }

    pub fn process(&self, loudness: f32, bass_energy: f32) -> ControllerOutput {
        let is_drop = bass_energy > self.config.drop_detection_threshold && loudness > 0.7;

        ControllerOutput { is_drop, loudness }
    }
}

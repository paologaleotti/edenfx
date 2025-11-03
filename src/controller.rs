use crate::{audio::AudioMetrics, config::AudioConfig};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default, Debug)]
pub struct ControllerOutput {
    pub is_drop: bool,
    pub loudness: f32,
}

pub struct Controller {
    config: Arc<Mutex<AudioConfig>>,
}

impl Controller {
    pub fn new(config: Arc<Mutex<AudioConfig>>) -> Self {
        Self { config }
    }

    pub fn process(&self, metrics: AudioMetrics) -> ControllerOutput {
        let threshold = self.config.lock().unwrap().drop_detection_threshold;
        let is_drop = metrics.bass_energy > threshold && metrics.loudness > 0.7;

        ControllerOutput {
            is_drop,
            loudness: metrics.loudness,
        }
    }
}

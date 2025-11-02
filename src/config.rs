pub const APP_VERSION: &str = "v0.0.1";

#[derive(Clone, PartialEq, Debug)]
pub struct AudioConfig {
    /// Sample rate in Hz. Standard CD quality is 44100 Hz.
    /// Higher = better frequency resolution but more CPU usage.
    pub sample_rate: f32,

    /// Number of samples to analyze at once. Must be a power of 2 for FFT.
    /// Larger = better frequency resolution but more latency.
    /// 2048 samples at 44.1kHz = ~46ms of audio
    pub buffer_size: usize,

    /// Maximum frequency (in Hz) considered as "bass".
    /// Typical ranges: Sub-bass (20-60Hz), Bass (60-250Hz)
    /// Lower values = only deep bass, Higher values = include more mid-bass
    pub bass_freq_max: f32,

    /// Multiplier to amplify bass energy readings.
    /// Higher = more sensitive to bass (drops will trigger easier)
    /// Lower = only very heavy bass will show high values
    pub bass_energy_multiplier: f32,

    /// Multiplier to amplify bass energy readings.
    /// Higher = more sensitive to bass (drops will trigger easier)
    /// Lower = only very heavy bass will show high values
    pub drop_detection_threshold: f32,

    /// Multiplier for overall loudness calculation.
    /// Higher = more sensitive to quiet sounds
    /// Lower = only loud sounds register high values
    pub loudness_multiplier: f32,

    /// Update interval in milliseconds for printing analysis to console.
    pub update_interval_ms: u64,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            buffer_size: 2048,
            bass_freq_max: 250.0,
            bass_energy_multiplier: 2.5,
            drop_detection_threshold: 0.8,
            loudness_multiplier: 10.0,
            update_interval_ms: 50,
        }
    }
}

/// Sample rate in Hz. Standard CD quality is 44100 Hz.
/// Higher = better frequency resolution but more CPU usage.
pub const SAMPLE_RATE: f32 = 44100.0;

/// Number of samples to analyze at once. Must be a power of 2 for FFT.
/// Larger = better frequency resolution but more latency.
/// 2048 samples at 44.1kHz = ~46ms of audio
pub const BUFFER_SIZE: usize = 2048;

/// Maximum frequency (in Hz) considered as "bass".
/// Typical ranges: Sub-bass (20-60Hz), Bass (60-250Hz)
/// Lower values = only deep bass, Higher values = include more mid-bass
pub const BASS_FREQ_MAX: f32 = 250.0;

/// Multiplier to amplify bass energy readings.
/// Higher = more sensitive to bass (drops will trigger easier)
/// Lower = only very heavy bass will show high values
/// Recommended range: 1.5 - 4.0
pub const BASS_ENERGY_MULTIPLIER: f32 = 2.5;

/// Threshold (0.0 - 1.0) for detecting bass drops.
/// When bass_energy exceeds this value, a drop is detected.
/// Lower = more sensitive (triggers more often)
/// Higher = less sensitive (only heavy drops)
/// Recommended range: 0.6 - 0.8
pub const DROP_DETECTION_THRESHOLD: f32 = 0.8;

/// Multiplier for overall loudness calculation.
/// Higher = more sensitive to quiet sounds
/// Lower = only loud sounds register high values
/// Recommended range: 5.0 - 15.0
pub const LOUDNESS_MULTIPLIER: f32 = 10.0;

/// Update interval in milliseconds for printing analysis to console.
/// Lower = more frequent updates (smoother but more CPU)
/// Higher = less frequent updates (choppier but less CPU)
/// Recommended range: 50 - 200ms
pub const UPDATE_INTERVAL_MS: u64 = 100;

use crate::config::AudioConfig;
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default, Debug)]
pub struct AudioMetrics {
    pub loudness: f32,
    pub bass_energy: f32,
}

pub struct AudioAnalyzer {
    config: Arc<Mutex<AudioConfig>>,
    buffer: Vec<f32>,
    fft_planner: FftPlanner<f32>,
}

impl AudioAnalyzer {
    pub fn new(config: Arc<Mutex<AudioConfig>>) -> Self {
        let buffer_size = config.lock().unwrap().buffer_size;
        Self {
            buffer: Vec::with_capacity(buffer_size),
            fft_planner: FftPlanner::new(),
            config,
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        // Keep only the most recent samples
        let buffer_size = self.config.lock().unwrap().buffer_size;
        if self.buffer.len() > buffer_size {
            self.buffer
                .drain(0..self.buffer.len() - buffer_size);
        }
    }

    pub fn calculate_loudness(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        // RMS (Root Mean Square) for loudness
        let sum_squares: f32 = self.buffer.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / self.buffer.len() as f32).sqrt();

        // Convert to 0-1 scale
        let loudness_multiplier = self.config.lock().unwrap().loudness_multiplier;
        (rms * loudness_multiplier).min(1.0)
    }

    pub fn calculate_bass_energy(&mut self) -> f32 {
        let config = self.config.lock().unwrap();
        let buffer_size = config.buffer_size;
        let sample_rate = config.sample_rate;
        let bass_freq_max = config.bass_freq_max;
        let bass_energy_multiplier = config.bass_energy_multiplier;
        drop(config); // Release lock early

        if self.buffer.len() < buffer_size {
            return 0.0;
        }

        // Prepare data for FFT
        let mut complex_buffer: Vec<Complex<f32>> = self.buffer
            [self.buffer.len() - buffer_size..]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Perform FFT
        let fft = self.fft_planner.plan_fft_forward(buffer_size);
        fft.process(&mut complex_buffer);

        // Calculate which FFT bin corresponds to our bass cutoff frequency
        let bin_freq = sample_rate / buffer_size as f32;
        let bass_bin_max = (bass_freq_max / bin_freq) as usize;

        // Ensure we have at least one bin to analyze (need at least 2 for range [1..2])
        if bass_bin_max < 2 {
            return 0.0;
        }

        // Calculate bass energy (sum of magnitudes in bass range)
        let bass_energy: f32 = complex_buffer[1..bass_bin_max.min(buffer_size / 2)]
            .iter()
            .map(|c| c.norm())
            .sum();

        // Calculate total energy for normalization
        let total_energy: f32 = complex_buffer[1..buffer_size / 2]
            .iter()
            .map(|c| c.norm())
            .sum();

        // Return normalized bass energy (0-1 range)
        if total_energy > 0.0 {
            let normalized = bass_energy / total_energy;
            // Scale it up to make drops more obvious
            (normalized * bass_energy_multiplier).min(1.0)
        } else {
            0.0
        }
    }

    pub fn analyze(&mut self) -> AudioMetrics {
        let loudness = self.calculate_loudness();
        let bass_energy = self.calculate_bass_energy();

        AudioMetrics {
            loudness,
            bass_energy,
        }
    }
}

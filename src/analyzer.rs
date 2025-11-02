use rustfft::{FftPlanner, num_complex::Complex};

use crate::consts;

pub struct AudioAnalyzer {
    buffer: Vec<f32>,
    fft_planner: FftPlanner<f32>,
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(consts::BUFFER_SIZE),
            fft_planner: FftPlanner::new(),
        }
    }

    pub fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        // Keep only the most recent samples
        if self.buffer.len() > consts::BUFFER_SIZE {
            self.buffer
                .drain(0..self.buffer.len() - consts::BUFFER_SIZE);
        }
    }

    pub fn calculate_loudness(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        // RMS (Root Mean Square) for loudness
        let sum_squares: f32 = self.buffer.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / self.buffer.len() as f32).sqrt();

        // Convert to a more intuitive 0-1 scale
        (rms * consts::LOUDNESS_MULTIPLIER).min(1.0)
    }

    pub fn calculate_bass_energy(&mut self) -> f32 {
        if self.buffer.len() < consts::BUFFER_SIZE {
            return 0.0;
        }

        // Prepare data for FFT
        let mut complex_buffer: Vec<Complex<f32>> = self.buffer
            [self.buffer.len() - consts::BUFFER_SIZE..]
            .iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();

        // Perform FFT
        let fft = self.fft_planner.plan_fft_forward(consts::BUFFER_SIZE);
        fft.process(&mut complex_buffer);

        // Calculate which FFT bin corresponds to our bass cutoff frequency
        let bin_freq = consts::SAMPLE_RATE / consts::BUFFER_SIZE as f32;
        let bass_bin_max = (consts::BASS_FREQ_MAX / bin_freq) as usize;

        // Calculate bass energy (sum of magnitudes in bass range)
        let bass_energy: f32 = complex_buffer[1..bass_bin_max.min(consts::BUFFER_SIZE / 2)]
            .iter()
            .map(|c| c.norm())
            .sum();

        // Calculate total energy for normalization
        let total_energy: f32 = complex_buffer[1..consts::BUFFER_SIZE / 2]
            .iter()
            .map(|c| c.norm())
            .sum();

        // Return normalized bass energy (0-1 range)
        if total_energy > 0.0 {
            let normalized = bass_energy / total_energy;
            // Scale it up to make drops more obvious
            (normalized * consts::BASS_ENERGY_MULTIPLIER).min(1.0)
        } else {
            0.0
        }
    }
}

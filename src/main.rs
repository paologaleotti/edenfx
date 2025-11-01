mod consts;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};

// FFT for frequency analysis
use rustfft::{FftPlanner, num_complex::Complex};

struct AudioAnalyzer {
    buffer: Vec<f32>,
    fft_planner: FftPlanner<f32>,
}

impl AudioAnalyzer {
    fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(consts::BUFFER_SIZE),
            fft_planner: FftPlanner::new(),
        }
    }

    fn add_samples(&mut self, samples: &[f32]) {
        self.buffer.extend_from_slice(samples);

        // Keep only the most recent samples
        if self.buffer.len() > consts::BUFFER_SIZE {
            self.buffer
                .drain(0..self.buffer.len() - consts::BUFFER_SIZE);
        }
    }

    fn calculate_loudness(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }

        // RMS (Root Mean Square) for loudness
        let sum_squares: f32 = self.buffer.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / self.buffer.len() as f32).sqrt();

        // Convert to a more intuitive 0-1 scale
        (rms * consts::LOUDNESS_MULTIPLIER).min(1.0)
    }

    fn calculate_bass_energy(&mut self) -> f32 {
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

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    // List all available input devices
    let devices: Vec<cpal::Device> = host.input_devices()?.collect();

    if devices.is_empty() {
        eprintln!("No input devices found!");
        return Ok(());
    }

    println!("Available audio input devices:");
    println!("-------------------------------");
    for (i, device) in devices.iter().enumerate() {
        println!(
            "{}. {}",
            i,
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );
    }
    println!("-------------------------------");

    println!("\nEnter device number to use: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let device_index: usize = input.trim().parse().expect("Please enter a valid number");

    let device = devices.get(device_index).expect("Invalid device number");

    println!("\n✓ Using device: {}", device.name()?);

    let config = device.default_input_config()?;
    println!("Config: {:?}\n", config);

    let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new()));
    let analyzer_clone = analyzer.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &config.into(), analyzer_clone)?,
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &config.into(), analyzer_clone)?,
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &config.into(), analyzer_clone)?,
        _ => panic!("Unsupported sample format"),
    };

    stream.play()?;

    println!("\nListening to audio... (Press Ctrl+C to stop)\n");
    println!("Loudness | Bass Energy | Status");
    println!("---------|-------------|--------");

    loop {
        std::thread::sleep(std::time::Duration::from_millis(consts::UPDATE_INTERVAL_MS));

        let mut analyzer = analyzer.lock().unwrap();
        let loudness = analyzer.calculate_loudness();
        let bass_energy = analyzer.calculate_bass_energy();

        let drop_detected = bass_energy > consts::DROP_DETECTION_THRESHOLD;

        print!(
            "\r{:>7.2}% | {:>10.2}% | {}",
            loudness * 100.0,
            bass_energy * 100.0,
            if drop_detected { "DROP!" } else { "       " }
        );

        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    analyzer: Arc<Mutex<AudioAnalyzer>>,
) -> Result<cpal::Stream, anyhow::Error>
where
    T: Sample + FromSample<f32> + cpal::SizedSample,
    f32: FromSample<T>,
{
    let err_fn = |err| eprintln!("Error on stream: {}", err);

    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let samples: Vec<f32> = data.iter().map(|&s| s.to_sample()).collect();
            let mut analyzer = analyzer.lock().unwrap();
            analyzer.add_samples(&samples);
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

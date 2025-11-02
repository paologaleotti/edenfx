mod analyzer;
mod consts;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};

use crate::analyzer::AudioAnalyzer;

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();

    let audio_devices: Vec<cpal::Device> = host.input_devices()?.collect();

    if audio_devices.is_empty() {
        eprintln!("No input devices found!");
        return Ok(());
    }

    println!("Available audio input devices:");
    println!("-------------------------------");
    for (i, device) in audio_devices.iter().enumerate() {
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

    let device = audio_devices
        .get(device_index)
        .expect("Invalid device number");

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

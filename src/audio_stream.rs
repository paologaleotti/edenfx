use crate::analyzer::{AudioAnalyzer, AudioMetrics};
use crate::config::AudioConfig;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat};
use std::sync::{Arc, Mutex};

pub struct AudioStream {
    _stream: Option<cpal::Stream>,
    analyzer: Arc<Mutex<AudioAnalyzer>>,
}

impl AudioStream {
    pub fn new(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sample_format: SampleFormat,
        audio_config: AudioConfig,
    ) -> Result<Self, anyhow::Error> {
        let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new(audio_config)));
        let analyzer_clone = analyzer.clone();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(device, config, analyzer_clone)?,
            cpal::SampleFormat::I16 => build_stream::<i16>(device, config, analyzer_clone)?,
            cpal::SampleFormat::U16 => build_stream::<u16>(device, config, analyzer_clone)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self {
            _stream: Some(stream),
            analyzer,
        })
    }

    pub fn get_metrics(&self) -> AudioMetrics {
        self.analyzer.lock().unwrap().analyze()
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
    let stream = device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let samples: Vec<f32> = data.iter().map(|&s| s.to_sample()).collect();
            analyzer.lock().unwrap().add_samples(&samples);
        },
        |err| eprintln!("Stream error: {}", err),
        None,
    )?;

    Ok(stream)
}

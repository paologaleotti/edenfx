use crate::analyzer::AudioAnalyzer;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat};
use std::sync::{Arc, Mutex};

pub struct AudioStream {
    _stream: cpal::Stream,
}

impl AudioStream {
    pub fn new(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sample_format: SampleFormat,
        analyzer: Arc<Mutex<AudioAnalyzer>>,
    ) -> Result<Self, anyhow::Error> {
        let stream = match sample_format {
            cpal::SampleFormat::F32 => build_stream::<f32>(device, config, analyzer)?,
            cpal::SampleFormat::I16 => build_stream::<i16>(device, config, analyzer)?,
            cpal::SampleFormat::U16 => build_stream::<u16>(device, config, analyzer)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play()?;

        Ok(Self { _stream: stream })
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
        |err| eprintln!("Stream error: {err}"),
        None,
    )?;

    Ok(stream)
}

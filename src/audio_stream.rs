use crate::analyzer::AudioAnalyzer;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
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

pub fn create_audio_stream(
    device_idx: usize,
    devices: &[String],
    analyzer: Arc<Mutex<AudioAnalyzer>>,
) -> Option<AudioStream> {
    let host = cpal::default_host();

    // Get the device by matching name
    let device = if let Some(device_name) = devices.get(device_idx) {
        host.input_devices()
            .ok()?
            .find(|d| d.name().ok().as_ref() == Some(device_name))
    } else {
        None
    }?;

    let supported_config = device.default_input_config().ok()?;
    let sample_format = supported_config.sample_format();
    let stream_config: cpal::StreamConfig = supported_config.into();

    AudioStream::new(&device, &stream_config, sample_format, analyzer).ok()
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

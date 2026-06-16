use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

/// Records the default input device to a mono 16-bit WAV file.
/// The cpal stream lives on a dedicated thread because it is `!Send`.
pub struct Recorder {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<anyhow::Result<PathBuf>>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            stop: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.handle.is_some()
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        if self.handle.is_some() {
            return Ok(());
        }
        self.stop.store(false, Ordering::SeqCst);
        let stop = self.stop.clone();
        self.handle = Some(std::thread::spawn(move || record_loop(stop)));
        Ok(())
    }

    /// Stops capture, finalizes the WAV, and returns its path.
    pub fn stop(&mut self) -> anyhow::Result<Option<PathBuf>> {
        self.stop.store(true, Ordering::SeqCst);
        match self.handle.take() {
            Some(handle) => match handle.join() {
                Ok(res) => res.map(Some),
                Err(_) => Err(anyhow::anyhow!("recording thread panicked")),
            },
            None => Ok(None),
        }
    }
}

fn record_loop(stop: Arc<AtomicBool>) -> anyhow::Result<PathBuf> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow::anyhow!("no default input device"))?;

    let supported = device.default_input_config()?;
    let sample_format = supported.sample_format();
    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    let stream_config: cpal::StreamConfig = supported.into();

    let samples: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    let err_fn = |e| eprintln!("audio stream error: {e}");

    let stream = match sample_format {
        SampleFormat::F32 => {
            let buf = samples.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    let mut b = buf.lock().unwrap();
                    for frame in data.chunks(channels) {
                        let sum: f32 = frame.iter().copied().sum();
                        let mono = sum / channels as f32;
                        b.push((mono.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
                    }
                },
                err_fn,
                None,
            )?
        }
        SampleFormat::I16 => {
            let buf = samples.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _| {
                    let mut b = buf.lock().unwrap();
                    for frame in data.chunks(channels) {
                        let sum: i32 = frame.iter().map(|&s| s as i32).sum();
                        b.push((sum / channels as i32) as i16);
                    }
                },
                err_fn,
                None,
            )?
        }
        SampleFormat::U16 => {
            let buf = samples.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _| {
                    let mut b = buf.lock().unwrap();
                    for frame in data.chunks(channels) {
                        let sum: i32 = frame.iter().map(|&s| s as i32 - 32768).sum();
                        b.push((sum / channels as i32) as i16);
                    }
                },
                err_fn,
                None,
            )?
        }
        other => return Err(anyhow::anyhow!("unsupported sample format: {other:?}")),
    };

    stream.play()?;
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(40));
    }
    drop(stream); // stop capture

    let mut path = std::env::temp_dir();
    path.push(format!("tpk-whisper-{}.wav", std::process::id()));

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec)?;
    for &s in samples.lock().unwrap().iter() {
        writer.write_sample(s)?;
    }
    writer.finalize()?;
    Ok(path)
}

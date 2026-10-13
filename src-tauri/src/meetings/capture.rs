//! Capture the microphone and the computer's own audio as mono 16 kHz.
//!
//! "Computer audio" is whatever plays through the default speakers or
//! headphones, so it works with Zoom, Meet, Teams, Slack huddles and browser
//! calls without any bot joining the call. On Windows and macOS (14.6+) this is
//! a loopback stream on the default output device. On Linux it is a
//! `*.monitor` input source.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, Sample, SampleFormat, SizedSample};
use log::{debug, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

pub const TARGET_RATE: u32 = 16_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    /// The user's microphone.
    You,
    /// Everything the computer plays (the other people on the call).
    Others,
}

impl Track {
    pub fn as_str(self) -> &'static str {
        match self {
            Track::You => "you",
            Track::Others => "others",
        }
    }
}

pub type AudioTx = Sender<(Track, Vec<f32>)>;

/// Running capture. Dropping or calling `stop` ends the stream.
pub struct CaptureHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl CaptureHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for CaptureHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

/// Averages groups of input samples down to 16 kHz. A box filter is crude but
/// removes most aliasing, which is plenty for speech recognition.
struct Downsampler {
    step: f64,
    phase: f64,
    acc: f32,
    count: u32,
}

impl Downsampler {
    fn new(input_rate: u32) -> Self {
        Self {
            step: (input_rate as f64 / TARGET_RATE as f64).max(1.0),
            phase: 0.0,
            acc: 0.0,
            count: 0,
        }
    }

    fn process(&mut self, mono: &[f32]) -> Vec<f32> {
        let mut out = Vec::with_capacity((mono.len() as f64 / self.step) as usize + 1);
        for &s in mono {
            self.acc += s;
            self.count += 1;
            self.phase += 1.0;
            if self.phase >= self.step {
                out.push(self.acc / self.count as f32);
                self.acc = 0.0;
                self.count = 0;
                self.phase -= self.step;
            }
        }
        out
    }
}

fn build_stream<T>(
    device: &Device,
    config: cpal::StreamConfig,
    channels: usize,
    rate: u32,
    track: Track,
    tx: AudioTx,
) -> Result<cpal::Stream, String>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let mut down = Downsampler::new(rate);
    let channels = channels.max(1);
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &_| {
                let mut mono = Vec::with_capacity(data.len() / channels + 1);
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
                    mono.push(sum / frame.len() as f32);
                }
                let out = down.process(&mono);
                if !out.is_empty() {
                    let _ = tx.send((track, out));
                }
            },
            move |err| warn!("Meeting audio stream error ({}): {}", track.as_str(), err),
            None,
        )
        .map_err(|e| e.to_string())
}

fn device_name(device: &Device) -> String {
    device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unknown device".to_string())
}

fn pick_microphone(preferred: Option<&str>) -> Option<Device> {
    let host = cpal::default_host();
    if let Some(name) = preferred.filter(|n| !n.is_empty() && *n != "Default") {
        if let Ok(devices) = host.input_devices() {
            for device in devices {
                if device_name(&device) == name {
                    return Some(device);
                }
            }
        }
    }
    host.default_input_device()
}

/// The device whose audio is "what the computer plays".
fn pick_system_audio() -> Result<(Device, bool), String> {
    let host = cpal::default_host();

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        // A loopback stream is an input stream opened on the output device.
        let device = host
            .default_output_device()
            .ok_or_else(|| "No speakers or headphones found".to_string())?;
        Ok((device, true))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let devices = host.input_devices().map_err(|e| e.to_string())?;
        for device in devices {
            if device_name(&device).to_lowercase().contains("monitor") {
                return Ok((device, false));
            }
        }
        Err("No monitor source found. Computer audio capture needs PulseAudio or PipeWire.".into())
    }
}

/// Start capturing one track. The stream lives on its own thread because some
/// platform streams cannot move between threads.
pub fn start(
    track: Track,
    preferred_mic: Option<String>,
    tx: AudioTx,
) -> Result<CaptureHandle, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

    let thread = std::thread::Builder::new()
        .name(format!("meeting-capture-{}", track.as_str()))
        .spawn(move || {
            let built = (|| -> Result<cpal::Stream, String> {
                let (device, is_output) = match track {
                    Track::You => (
                        pick_microphone(preferred_mic.as_deref())
                            .ok_or_else(|| "No microphone found".to_string())?,
                        false,
                    ),
                    Track::Others => pick_system_audio()?,
                };
                let supported = if is_output {
                    device.default_output_config()
                } else {
                    device.default_input_config()
                }
                .map_err(|e| e.to_string())?;
                debug!(
                    "Meeting capture {}: {} ({:?}, {} ch, {} Hz)",
                    track.as_str(),
                    device_name(&device),
                    supported.sample_format(),
                    supported.channels(),
                    supported.sample_rate()
                );
                let format = supported.sample_format();
                let channels = supported.channels() as usize;
                let rate = supported.sample_rate() as u32;
                let config: cpal::StreamConfig = supported.into();
                let stream = match format {
                    SampleFormat::F32 => {
                        build_stream::<f32>(&device, config, channels, rate, track, tx)
                    }
                    SampleFormat::I16 => {
                        build_stream::<i16>(&device, config, channels, rate, track, tx)
                    }
                    SampleFormat::I32 => {
                        build_stream::<i32>(&device, config, channels, rate, track, tx)
                    }
                    SampleFormat::U16 => {
                        build_stream::<u16>(&device, config, channels, rate, track, tx)
                    }
                    other => Err(format!("Unsupported audio format: {other}")),
                }?;
                stream.play().map_err(|e| e.to_string())?;
                Ok(stream)
            })();

            match built {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    while !stop_thread.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        })
        .map_err(|e| e.to_string())?;

    match ready_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(())) => Ok(CaptureHandle {
            stop,
            thread: Some(thread),
        }),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(e)
        }
        Err(_) => {
            stop.store(true, Ordering::SeqCst);
            Err("Audio device did not respond".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_48k_to_16k_keeps_length_ratio() {
        let mut d = Downsampler::new(48_000);
        let input = vec![0.5f32; 4_800];
        let out = d.process(&input);
        assert_eq!(out.len(), 1_600);
        assert!(out.iter().all(|s| (*s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn downsample_is_continuous_across_chunks() {
        let mut d = Downsampler::new(44_100);
        let a = d.process(&vec![0.25f32; 1_000]);
        let b = d.process(&vec![0.25f32; 1_000]);
        let expected = (2_000.0f64 / (44_100.0 / 16_000.0)) as usize;
        assert!((a.len() + b.len()).abs_diff(expected) <= 1);
    }

    #[test]
    fn same_rate_passes_through() {
        let mut d = Downsampler::new(16_000);
        let out = d.process(&[0.1, 0.2, 0.3]);
        assert_eq!(out, vec![0.1, 0.2, 0.3]);
    }
}

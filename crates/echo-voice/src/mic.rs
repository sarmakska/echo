//! Real microphone capture via cpal. Behind the `mic` feature so the default
//! build needs no audio system libraries. Needs an input device at runtime.
#![cfg(feature = "mic")]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::traits::MicSource;

/// Captures a fixed window of audio from the default input device, downmixed to
/// mono 16-bit. Fixed-window capture is the Phase 1 baseline; VAD end-pointing is
/// a later refinement (the `EnergyVadWakeWord` already provides energy gating).
pub struct CpalMic {
    capture: Duration,
}

impl CpalMic {
    pub fn new(capture_ms: u64) -> Self {
        Self { capture: Duration::from_millis(capture_ms) }
    }
}

impl MicSource for CpalMic {
    fn next_utterance(&mut self) -> Option<Vec<i16>> {
        let host = cpal::default_host();
        let device = host.default_input_device()?;
        let supported = device.default_input_config().ok()?;
        let sample_format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();

        let buffer = Arc::new(Mutex::new(Vec::<i16>::new()));
        let err_fn = |e| eprintln!("echo-voice cpal stream error: {e}");

        let sink = buffer.clone();
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut g = sink.lock().unwrap();
                        g.extend(data.iter().map(|&s| (s * i16::MAX as f32) as i16));
                    },
                    err_fn,
                    None,
                )
                .ok()?,
            cpal::SampleFormat::I16 => device
                .build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        sink.lock().unwrap().extend_from_slice(data);
                    },
                    err_fn,
                    None,
                )
                .ok()?,
            _ => return None,
        };

        stream.play().ok()?;
        std::thread::sleep(self.capture);
        drop(stream);

        let captured = buffer.lock().unwrap().clone();
        Some(captured)
    }
}

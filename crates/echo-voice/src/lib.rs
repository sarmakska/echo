//! Echo voice loop. Trait contracts + fakes + the turn-driving `VoiceLoop` with
//! barge-in. Real backends: `EnergyVadWakeWord` (default, dependency-free wake),
//! `WhisperCliStt` (whisper.cpp), `PiperTts` (Piper), `SayTts` (macOS), and the
//! feature-gated `CpalMic` (`mic`) and `PorcupineWakeWord` (`wake`).

mod fakes;
#[cfg(feature = "mic")]
mod mic;
mod piper;
#[cfg(feature = "wake")]
mod porcupine_wake;
#[cfg(target_os = "macos")]
mod say;
mod sentences;
mod traits;
mod vad;
mod voiceloop;
mod wav;
mod whisper_cli;

pub use fakes::{FakeMic, FakeStt, FakeTts, FakeWakeWord};
#[cfg(feature = "mic")]
pub use mic::CpalMic;
pub use piper::{build_piper_args, PiperTts};
#[cfg(feature = "wake")]
pub use porcupine_wake::PorcupineWakeWord;
#[cfg(target_os = "macos")]
pub use say::{build_say_args, SayTts};
pub use sentences::split_sentences;
pub use traits::{MicSource, Stt, Tts, VoiceError, WakeWord};
pub use vad::{frame_rms, EnergyVadWakeWord};
pub use voiceloop::VoiceLoop;
pub use wav::encode_wav_mono16;
pub use whisper_cli::{build_whisper_args, WhisperCliStt};

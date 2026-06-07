#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("stt failed: {0}")]
    Stt(String),
    #[error("tts failed: {0}")]
    Tts(String),
    #[error("mic failed: {0}")]
    Mic(String),
}

/// Detects the wake word in a window of PCM samples. Real impl: Porcupine.
pub trait WakeWord {
    fn detect(&mut self, samples: &[i16]) -> bool;
}

/// Speech-to-text. Real impl: whisper.cpp.
pub trait Stt {
    fn transcribe(&self, samples: &[i16]) -> Result<String, VoiceError>;
}

/// Text-to-speech. Real impl: Piper / OS-native.
pub trait Tts {
    fn speak(&self, text: &str) -> Result<(), VoiceError>;
}

/// Source of captured utterances (post-VAD). Real impl: cpal capture thread.
pub trait MicSource {
    /// Next captured utterance as PCM, or None when the source is exhausted.
    fn next_utterance(&mut self) -> Option<Vec<i16>>;
}

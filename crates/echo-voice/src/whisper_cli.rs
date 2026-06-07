use std::fs;
use std::process::Command;

use crate::traits::{Stt, VoiceError};
use crate::wav::encode_wav_mono16;

/// Build the whisper.cpp CLI arguments. `-otxt -nt` writes a plain transcript
/// with no timestamps to `<wav>.txt`; `-nt` keeps stdout clean.
pub fn build_whisper_args(model_path: &str, wav_path: &str) -> Vec<String> {
    vec![
        "-m".to_string(),
        model_path.to_string(),
        "-f".to_string(),
        wav_path.to_string(),
        "-otxt".to_string(),
        "-nt".to_string(),
    ]
}

/// Speech-to-text via the whisper.cpp CLI binary (PLAN: whisper.cpp, models in
/// `~/.echo/models/`, default `small.en`). Writes the captured PCM to a temp WAV,
/// runs the binary, and reads the produced transcript file. Needs the binary and
/// a model present at runtime; the argument and WAV encoding are unit-tested.
pub struct WhisperCliStt {
    binary: String,
    model_path: String,
    sample_rate: u32,
}

impl WhisperCliStt {
    /// `binary` is typically `whisper-cli` (or the whisper.cpp `main`); `model_path`
    /// points at e.g. `~/.echo/models/ggml-small.en.bin`.
    pub fn new(binary: impl Into<String>, model_path: impl Into<String>) -> Self {
        Self { binary: binary.into(), model_path: model_path.into(), sample_rate: 16_000 }
    }
}

impl Stt for WhisperCliStt {
    fn transcribe(&self, samples: &[i16]) -> Result<String, VoiceError> {
        let wav_path = std::env::temp_dir().join("echo-stt-input.wav");
        let wav = encode_wav_mono16(samples, self.sample_rate);
        fs::write(&wav_path, &wav).map_err(|e| VoiceError::Stt(e.to_string()))?;

        let wav_str = wav_path.to_string_lossy().to_string();
        let status = Command::new(&self.binary)
            .args(build_whisper_args(&self.model_path, &wav_str))
            .status()
            .map_err(|e| VoiceError::Stt(format!("spawn {}: {e}", self.binary)))?;
        if !status.success() {
            return Err(VoiceError::Stt(format!("whisper exited with {:?}", status.code())));
        }

        let txt_path = format!("{wav_str}.txt");
        let text = fs::read_to_string(&txt_path).map_err(|e| VoiceError::Stt(e.to_string()))?;
        Ok(text.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_whisper_args() {
        let args = build_whisper_args("/m/ggml-small.en.bin", "/tmp/a.wav");
        assert_eq!(
            args,
            vec!["-m", "/m/ggml-small.en.bin", "-f", "/tmp/a.wav", "-otxt", "-nt"]
        );
    }
}

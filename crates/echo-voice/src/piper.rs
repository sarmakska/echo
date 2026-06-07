use std::io::Write;
use std::process::{Command, Stdio};

use crate::traits::{Tts, VoiceError};

/// Build Piper arguments: read text from stdin, synthesise to `out_wav`.
pub fn build_piper_args(model_path: &str, out_wav: &str) -> Vec<String> {
    vec![
        "--model".to_string(),
        model_path.to_string(),
        "--output_file".to_string(),
        out_wav.to_string(),
    ]
}

/// Default TTS (PLAN section 4.4): the Piper binary synthesises a WAV which a
/// platform player then plays (`afplay` on macOS, `aplay` on Linux, configurable
/// on Windows). Cross-platform code; needs the Piper binary, a voice model, and a
/// player present at runtime. Argument building is unit-tested.
pub struct PiperTts {
    piper_binary: String,
    model_path: String,
    player_binary: String,
}

impl PiperTts {
    pub fn new(
        piper_binary: impl Into<String>,
        model_path: impl Into<String>,
        player_binary: impl Into<String>,
    ) -> Self {
        Self {
            piper_binary: piper_binary.into(),
            model_path: model_path.into(),
            player_binary: player_binary.into(),
        }
    }

    /// macOS preset: `piper` + `afplay`, neutral British voice model path.
    pub fn macos(model_path: impl Into<String>) -> Self {
        Self::new("piper", model_path, "afplay")
    }
}

impl Tts for PiperTts {
    fn speak(&self, text: &str) -> Result<(), VoiceError> {
        let out_wav = std::env::temp_dir().join("echo-tts-out.wav");
        let out_str = out_wav.to_string_lossy().to_string();

        let mut child = Command::new(&self.piper_binary)
            .args(build_piper_args(&self.model_path, &out_str))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .map_err(|e| VoiceError::Tts(format!("spawn piper: {e}")))?;
        {
            let mut stdin = child
                .stdin
                .take()
                .ok_or_else(|| VoiceError::Tts("piper stdin unavailable".to_string()))?;
            stdin.write_all(text.as_bytes()).map_err(|e| VoiceError::Tts(e.to_string()))?;
        }
        let status = child.wait().map_err(|e| VoiceError::Tts(e.to_string()))?;
        if !status.success() {
            return Err(VoiceError::Tts(format!("piper exited with {:?}", status.code())));
        }

        let play = Command::new(&self.player_binary)
            .arg(&out_str)
            .status()
            .map_err(|e| VoiceError::Tts(format!("spawn {}: {e}", self.player_binary)))?;
        if !play.success() {
            return Err(VoiceError::Tts(format!("player exited with {:?}", play.code())));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_piper_args() {
        let args = build_piper_args("/voices/en_GB.onnx", "/tmp/out.wav");
        assert_eq!(args, vec!["--model", "/voices/en_GB.onnx", "--output_file", "/tmp/out.wav"]);
    }
}

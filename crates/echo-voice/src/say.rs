//! macOS text-to-speech via the built-in `say` command (zero install).
//! Compiled only on macOS; other platforms get their own adapters later.
#![cfg(target_os = "macos")]

use std::process::Command;

use crate::traits::{Tts, VoiceError};

/// Build the argument vector for the `say` binary. Pure, so it's unit-testable.
pub fn build_say_args(voice: Option<&str>, text: &str) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(v) = voice {
        args.push("-v".to_string());
        args.push(v.to_string());
    }
    args.push(text.to_string());
    args
}

/// Speaks via the macOS `say` command. `voice` is an optional system voice name
/// (e.g. "Daniel" for British English); None uses the system default.
pub struct SayTts {
    voice: Option<String>,
}

impl SayTts {
    pub fn new(voice: Option<String>) -> Self {
        Self { voice }
    }
}

impl Tts for SayTts {
    fn speak(&self, text: &str) -> Result<(), VoiceError> {
        let args = build_say_args(self.voice.as_deref(), text);
        let status = Command::new("say")
            .args(&args)
            .status()
            .map_err(|e| VoiceError::Tts(e.to_string()))?;
        if status.success() {
            Ok(())
        } else {
            Err(VoiceError::Tts(format!("say exited with {:?}", status.code())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_without_voice_are_just_the_text() {
        assert_eq!(build_say_args(None, "hello"), vec!["hello"]);
    }

    #[test]
    fn args_with_voice_prepend_v_flag() {
        assert_eq!(build_say_args(Some("Daniel"), "hi"), vec!["-v", "Daniel", "hi"]);
    }
}

//! Real Porcupine wake word (PLAN: Picovoice, ~30ms latency, free personal tier).
//! Behind the `wake` feature because it needs a Picovoice access key and the
//! native Porcupine library at runtime. The dependency-free `EnergyVadWakeWord`
//! is the default until a key is configured.
//!
//! NOTE: targets the `porcupine` crate (Picovoice Rust SDK, 0.2.x) documented
//! API. It cannot be compiled in the headless build environment (no access key
//! or native library), so confirm against the installed version before relying
//! on it.
#![cfg(feature = "wake")]

use porcupine::{BuiltinKeywords, Porcupine, PorcupineBuilder};

use crate::traits::WakeWord;

fn keyword_for(name: &str) -> BuiltinKeywords {
    match name.to_ascii_lowercase().as_str() {
        "computer" => BuiltinKeywords::Computer,
        "alexa" => BuiltinKeywords::Alexa,
        "hey google" | "hey-google" => BuiltinKeywords::HeyGoogle,
        // Default and "echo" map to "jarvis" until a custom "echo" .ppn is trained.
        _ => BuiltinKeywords::Jarvis,
    }
}

/// Porcupine-backed wake word.
pub struct PorcupineWakeWord {
    engine: Porcupine,
}

impl PorcupineWakeWord {
    /// Build from a Picovoice access key and a built-in keyword name
    /// (e.g. "jarvis", "computer"). "echo" falls back to "jarvis" until a custom
    /// keyword model is trained.
    pub fn from_keyword(access_key: &str, keyword: &str) -> Result<Self, String> {
        let engine = PorcupineBuilder::new_with_keywords(access_key, &[keyword_for(keyword)])
            .init()
            .map_err(|e| format!("porcupine init: {e:?}"))?;
        Ok(Self { engine })
    }

    /// Samples Porcupine expects per `process` call.
    pub fn frame_length(&self) -> usize {
        self.engine.frame_length() as usize
    }
}

impl WakeWord for PorcupineWakeWord {
    fn detect(&mut self, samples: &[i16]) -> bool {
        // A non-negative keyword index means a wake word fired.
        matches!(self.engine.process(samples), Ok(index) if index >= 0)
    }
}

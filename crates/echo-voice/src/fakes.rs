use std::cell::RefCell;

use crate::traits::{MicSource, Stt, Tts, VoiceError, WakeWord};

/// Wake word that fires after a fixed number of `detect` calls.
pub struct FakeWakeWord {
    fires_after: usize,
    calls: usize,
}
impl FakeWakeWord {
    pub fn new(fires_after: usize) -> Self {
        Self { fires_after, calls: 0 }
    }
}
impl WakeWord for FakeWakeWord {
    fn detect(&mut self, _samples: &[i16]) -> bool {
        self.calls += 1;
        self.calls >= self.fires_after
    }
}

/// STT that returns a canned transcript regardless of audio.
pub struct FakeStt {
    transcript: String,
}
impl FakeStt {
    pub fn new(transcript: impl Into<String>) -> Self {
        Self { transcript: transcript.into() }
    }
}
impl Stt for FakeStt {
    fn transcribe(&self, _samples: &[i16]) -> Result<String, VoiceError> {
        Ok(self.transcript.clone())
    }
}

/// TTS that records everything it was asked to speak.
#[derive(Default)]
pub struct FakeTts {
    pub spoken: RefCell<Vec<String>>,
}
impl Tts for FakeTts {
    fn speak(&self, text: &str) -> Result<(), VoiceError> {
        self.spoken.borrow_mut().push(text.to_string());
        Ok(())
    }
}

/// Mic that yields a fixed list of utterances then stops.
pub struct FakeMic {
    utterances: std::vec::IntoIter<Vec<i16>>,
}
impl FakeMic {
    pub fn new(utterances: Vec<Vec<i16>>) -> Self {
        Self { utterances: utterances.into_iter() }
    }
}
impl MicSource for FakeMic {
    fn next_utterance(&mut self) -> Option<Vec<i16>> {
        self.utterances.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_wake_word_fires_after_n_calls() {
        let mut w = FakeWakeWord::new(3);
        assert!(!w.detect(&[]));
        assert!(!w.detect(&[]));
        assert!(w.detect(&[]));
    }

    #[test]
    fn fake_stt_returns_canned() {
        let stt = FakeStt::new("what is on today");
        assert_eq!(stt.transcribe(&[]).unwrap(), "what is on today");
    }

    #[test]
    fn fake_tts_records_spoken_text() {
        let tts = FakeTts::default();
        tts.speak("hello").unwrap();
        tts.speak("world").unwrap();
        assert_eq!(*tts.spoken.borrow(), vec!["hello", "world"]);
    }

    #[test]
    fn fake_mic_drains_then_stops() {
        let mut m = FakeMic::new(vec![vec![1, 2], vec![3]]);
        assert_eq!(m.next_utterance(), Some(vec![1, 2]));
        assert_eq!(m.next_utterance(), Some(vec![3]));
        assert_eq!(m.next_utterance(), None);
    }
}

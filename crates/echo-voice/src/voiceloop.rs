use crate::sentences::split_sentences;
use crate::traits::{MicSource, Stt, Tts, VoiceError, WakeWord};

/// Drives the full voice turn (PLAN §4): wake → capture → STT → handler → TTS,
/// with barge-in. Generic over the trait backends so it is fully testable with
/// fakes; real backends (VAD/Porcupine, cpal, whisper, Piper) plug in unchanged.
pub struct VoiceLoop<W: WakeWord, M: MicSource, S: Stt, T: Tts> {
    pub wake: W,
    pub mic: M,
    pub stt: S,
    pub tts: T,
}

impl<W: WakeWord, M: MicSource, S: Stt, T: Tts> VoiceLoop<W, M, S, T> {
    pub fn new(wake: W, mic: M, stt: S, tts: T) -> Self {
        Self { wake, mic, stt, tts }
    }

    /// Speak a reply sentence-by-sentence (PLAN §4.4 streaming TTS). Before each
    /// sentence, `interrupted()` is checked — if true, speaking stops (barge-in,
    /// PLAN §4.5). Returns how many sentences were actually spoken.
    pub fn speak_response(
        &self,
        reply: &str,
        mut interrupted: impl FnMut() -> bool,
    ) -> Result<usize, VoiceError> {
        let mut spoken = 0;
        for sentence in split_sentences(reply) {
            if interrupted() {
                break;
            }
            self.tts.speak(&sentence)?;
            spoken += 1;
        }
        Ok(spoken)
    }

    /// Handle one armed turn: capture an utterance, transcribe, hand the text to
    /// `handler`, speak the reply (with barge-in). Returns (transcript, reply),
    /// or None if the mic is exhausted.
    pub fn run_turn(
        &mut self,
        handler: &mut impl FnMut(&str) -> String,
        interrupted: impl FnMut() -> bool,
    ) -> Result<Option<(String, String)>, VoiceError> {
        let Some(samples) = self.mic.next_utterance() else {
            return Ok(None);
        };
        let transcript = self.stt.transcribe(&samples)?;
        let reply = handler(&transcript);
        self.speak_response(&reply, interrupted)?;
        Ok(Some((transcript, reply)))
    }

    /// Run until the mic is exhausted: pull frames, and on a wake-word hit handle
    /// one turn. Returns the number of turns handled. (Real deployments run this
    /// on a dedicated worker thread; tests drive a finite fake mic.)
    pub fn run(&mut self, handler: &mut impl FnMut(&str) -> String) -> Result<usize, VoiceError> {
        let mut turns = 0;
        while let Some(frame) = self.mic.next_utterance() {
            if self.wake.detect(&frame) && self.run_turn(handler, || false)?.is_some() {
                turns += 1;
            }
        }
        Ok(turns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fakes::{FakeMic, FakeStt, FakeTts, FakeWakeWord};
    use std::cell::Cell;

    #[test]
    fn run_handles_a_full_turn_after_wake() {
        // Frame 1 trips the wake word; frame 2 is the captured command audio.
        let mic = FakeMic::new(vec![vec![0, 0], vec![1, 2, 3]]);
        let mut vl = VoiceLoop::new(
            FakeWakeWord::new(1),
            mic,
            FakeStt::new("what is on today"),
            FakeTts::default(),
        );
        let mut handler = |t: &str| {
            assert_eq!(t, "what is on today");
            "Standup at 9:30.".to_string()
        };
        let turns = vl.run(&mut handler).unwrap();
        assert_eq!(turns, 1);
        assert_eq!(*vl.tts.spoken.borrow(), vec!["Standup at 9:30."]);
    }

    #[test]
    fn barge_in_stops_speaking_mid_reply() {
        let vl = VoiceLoop::new(
            FakeWakeWord::new(1),
            FakeMic::new(vec![]),
            FakeStt::new(""),
            FakeTts::default(),
        );
        // Interrupt fires before the 2nd sentence.
        let calls = Cell::new(0);
        let spoken = vl
            .speak_response("First. Second. Third.", || {
                let n = calls.get();
                calls.set(n + 1);
                n >= 1
            })
            .unwrap();
        assert_eq!(spoken, 1);
        assert_eq!(*vl.tts.spoken.borrow(), vec!["First."]);
    }
}

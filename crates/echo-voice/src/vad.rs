use crate::traits::WakeWord;

/// Root-mean-square amplitude of a PCM frame, normalised to 0.0..1.0.
pub fn frame_rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    let rms = (sum_sq / samples.len() as f64).sqrt();
    rms / i16::MAX as f64
}

/// Dependency-free wake fallback (PLAN section 4.1): fires when frame energy
/// exceeds `threshold` for `needed` consecutive frames. Requires no access key,
/// so it is the default until a Porcupine key is configured (see the `wake`
/// feature). Suitable for fully offline use.
pub struct EnergyVadWakeWord {
    threshold: f64,
    needed: u32,
    run: u32,
}

impl EnergyVadWakeWord {
    pub fn new(threshold: f64, needed: u32) -> Self {
        Self { threshold, needed: needed.max(1), run: 0 }
    }
}

impl Default for EnergyVadWakeWord {
    fn default() -> Self {
        // Sensible starting point; tuned per-mic during setup.
        Self::new(0.08, 2)
    }
}

impl WakeWord for EnergyVadWakeWord {
    fn detect(&mut self, samples: &[i16]) -> bool {
        if frame_rms(samples) >= self.threshold {
            self.run += 1;
        } else {
            self.run = 0;
        }
        if self.run >= self.needed {
            self.run = 0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loud(n: usize) -> Vec<i16> {
        vec![20_000; n]
    }
    fn quiet(n: usize) -> Vec<i16> {
        vec![10; n]
    }

    #[test]
    fn rms_of_silence_is_near_zero() {
        assert!(frame_rms(&[0, 0, 0]) < 0.001);
    }

    #[test]
    fn quiet_frames_never_fire() {
        let mut w = EnergyVadWakeWord::new(0.08, 2);
        assert!(!w.detect(&quiet(160)));
        assert!(!w.detect(&quiet(160)));
    }

    #[test]
    fn fires_after_needed_consecutive_loud_frames() {
        let mut w = EnergyVadWakeWord::new(0.08, 2);
        assert!(!w.detect(&loud(160))); // run = 1
        assert!(w.detect(&loud(160))); // run = 2 -> fire
    }

    #[test]
    fn a_quiet_frame_resets_the_run() {
        let mut w = EnergyVadWakeWord::new(0.08, 2);
        assert!(!w.detect(&loud(160)));
        assert!(!w.detect(&quiet(160))); // reset
        assert!(!w.detect(&loud(160))); // run = 1 again, no fire
    }
}

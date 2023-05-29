use std::default::default;

use crate::utils::sound::Note;
use crate::utils::synth::Waveform;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub amp: f64,
    pub phase: f64,
    pub detune: f64,

    pub t: f64,
    pub t0: f64,
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            waveform: default(),
            amp: 1.0,
            phase: 0.0,
            detune: 0.0,

            t: 0.0,
            t0: 0.0,
        }
    }
}

impl Oscillator {
    pub fn sample(&mut self, t: f64, note: Note) -> f64 {
        let dt = t - self.t;
        self.t = t;

        let freq = note.detune(self.detune).freq();

        self.t0 += dt * freq;
        self.waveform.sample(self.t0) * self.amp
    }
}

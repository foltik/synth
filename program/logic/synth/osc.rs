use std::default::default;

use crate::utils::sound::Note;
use crate::utils::synth::Waveform;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Osc {
    pub waveform: Waveform,

    pub amp: f64,
    pub phase: f64,
    pub tune: f64,

    pub phi: f64,
}

impl Default for Osc {
    fn default() -> Self {
        Self {
            waveform: default(),
            amp: 1.0,
            phase: 0.0,
            tune: 0.0,
            phi: 0.0,
        }
    }
}

impl Osc {
    pub fn sample(&mut self, dt: f64, note: Note) -> f64 {
        let freq = note.detune(self.tune).freq();
        self.phi += dt * freq;

        self.waveform.sample(self.phi + self.phase) * self.amp
    }
}

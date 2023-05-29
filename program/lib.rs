#![allow(unused)]
#![feature(default_free_fn)]
#![feature(lazy_cell)]

use std::default::default;
use std::f64::consts::PI;

use logic::synth;
use types::launchpad_x as lpx;
use types::launch_control_xl as lcx;

mod ffi;

mod utils;
use utils::*;
use utils::synth::Waveform;

use crate::utils::math::Float;

mod logic;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Program {
    piano: logic::pad::Piano,
    osc0: synth::Oscillator,
    osc1: synth::Oscillator,
    osc2: synth::Oscillator,
    volume: f64,
}

impl Default for Program {
    fn default() -> Self {
        Self {
            piano: default(),
            osc0: synth::Oscillator {
                waveform: Waveform::Saw,
                ..default()
            },
            osc1: synth::Oscillator {
                waveform: Waveform::Saw,
                ..default()
            },
            osc2: synth::Oscillator {
                waveform: Waveform::Saw,
                ..default()
            },
            volume: 0.01,
        }
    }
}

impl Program {
    pub fn pad_in(&mut self, t: f64, input: lpx::Input) {
        self.piano.pad_in(t, input);

        let ((x, y), v) = input;
        match (x, y) {
            (7, 8) if v > 0.0 => *self = Self::default(),
            _ => {},
            _ => {},
        }
    }
    pub fn pad_out(&mut self, t: f64) -> lpx::Output {
        let mut output = lpx::clear();
        self.piano.pad_out(t, &mut output);
        output
    }

    pub fn ctrl_in(&mut self, t: f64, input: lcx::Input) {
        use lcx::Input;
        println!("{input:?}");
        match input {
            Input::Knob(x, y, f) => match (x, y) {
                (0, 0) => self.osc0.detune = f,
                (1, 0) => self.osc1.detune = f,
                (2, 0) => self.osc2.detune = f,

                (0, 0) => self.osc0.detune = f * 12.0,
                (1, 0) => self.osc1.detune = f * 12.0,
                (2, 0) => self.osc2.detune = f * 12.0,

                (0, 2) => self.osc0.phase = f.mapf(),
                (1, 2) => self.osc1.phase = f.mapf(),
                (2, 2) => self.osc2.phase = f.mapf(),
                _ => {},
            },
            Input::Slider(i, f) => match i {
                0 => self.osc0.amp = f,
                1 => self.osc1.amp = f,
                2 => self.osc2.amp = f,

                7 => self.volume = f,
                _ => {},
            }
            _ => {},
        }
    }
    pub fn ctrl_out(&mut self, t: f64) -> lcx::Output { lcx::clear() }

    pub fn sample(&mut self, t: f64) -> (f64, f64) {
        let mut f = 0.0;

        for (note, v) in self.piano.notes() {
            f += self.osc0.sample(t, note) * v;
            f += self.osc1.sample(t, note) * v;
            f += self.osc2.sample(t, note) * v;
        }

        f *= self.volume;
        (f, f)
    }

    pub fn tick(&mut self, t: f64) {}
}

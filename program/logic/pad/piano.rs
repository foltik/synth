use std::default::default;

use types::launchpad_x as lpx;

use crate::utils::color::Color;
use crate::utils::sound::Note;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Piano {
    y: i8,
    octave: i8,
    inactive: Color,
    active: Color,
    notes: [f64; 13],
}

impl Default for Piano {
    fn default() -> Self {
        Self {
            y: 0,
            octave: 4,
            inactive: Color::white(),
            active: Color::blue(),
            notes: default(),
        }
    }
}

impl Piano {
    pub fn notes(&self) -> impl Iterator<Item = (Note, f64)> + '_ {
        self.notes.iter().copied().enumerate()
            .map(|(i, v)| (Note::from_midi(((self.octave + 1) * 12) + i as i8), v))
            .filter(|(_, v)| *v > 0.0)
    }

    pub fn pad_in(&mut self, _t: f64, ((x, mut y), fr): lpx::Input) {
        y -= self.y;
        match (x, y) {
            (0, 0) => self.notes[0] = fr,
            (1, 1) => self.notes[1] = fr,
            (1, 0) => self.notes[2] = fr,
            (2, 1) => self.notes[3] = fr,
            (2, 0) => self.notes[4] = fr,
            (3, 0) => self.notes[5] = fr,
            (4, 1) => self.notes[6] = fr,
            (4, 0) => self.notes[7] = fr,
            (5, 1) => self.notes[8] = fr,
            (5, 0) => self.notes[9] = fr,
            (6, 1) => self.notes[10] = fr,
            (6, 0) => self.notes[11] = fr,
            (7, 0) => self.notes[12] = fr,
            _ => {}
        }
    }

    pub fn pad_out(&self, _t: f64, output: &mut lpx::Output) {
        for (i, &v) in self.notes.iter().enumerate() {
            let (x, mut y) = match i {
                0 => (0, 0),
                1 => (1, 1),
                2 => (1, 0),
                3 => (2, 1),
                4 => (2, 0),
                5 => (3, 0),
                6 => (4, 1),
                7 => (4, 0),
                8 => (5, 1),
                9 => (5, 0),
                10 => (6, 1),
                11 => (6, 0),
                12 => (7, 0),
                _ => unreachable!(),
            };
            y += self.y;

            let color = if v > 0.0 { self.active } else { self.inactive }.into();
            output[(y * 9 + x) as usize] = color;
        }
    }
}

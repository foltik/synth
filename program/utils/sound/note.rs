#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Letter {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Accidental {
    DoubleFlat,
    Flat,
    Natural,
    Sharp,
    DoubleSharp,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Tuning {
    pub a4_freq: f64,
    pub temperament: Temperament,
}

impl Default for Tuning {
    fn default() -> Self {
        Self { a4_freq: 440.0, temperament: Temperament::Equal }
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Temperament {
    Equal,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Note {
    pub letter: Letter,
    pub accidental: Accidental,
    pub octave: i8,
    pub detune: f64,
}

impl Note {
    pub fn freq(&self) -> f64 {
        const TUNING: Tuning = Tuning { a4_freq: 440.0, temperament: Temperament::Equal };

        // scale the A4 note to self.octave
        let a_scaled = TUNING.a4_freq * 2.0f64.powi(self.octave as i32 - 4);

        // calculate the distance from the scaled A note
        let steps = self.letter.offset() + self.accidental.offset() - Letter::A.offset();

        match TUNING.temperament {
            Temperament::Equal => a_scaled * 2.0f64.powf((steps as f64 + self.detune) / 12.0)
        }
    }

    pub fn detune(self, semitones: f64) -> Self {
        Self { detune: self.detune + semitones, ..self }
    }

    // pub fn detune(&self, cents: f64) -> f64 {

    // }
}

impl Letter {
    fn offset(&self) -> isize {
        match self {
            Letter::C => 0,
            Letter::D => 2,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 7,
            Letter::A => 9,
            Letter::B => 11,
        }
    }
}

impl Accidental {
    fn offset(&self) -> isize {
        match self {
            Accidental::DoubleFlat => -2,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::DoubleSharp => 2,
        }
    }
}

impl Note {
    pub fn from_midi(midi: i8) -> Self {
        let octave = (midi / 12) - 2;
        let (letter, accidental) = match midi % 12 {
            0 => (Letter::C, Accidental::Natural),
            1 => (Letter::C, Accidental::Sharp),
            2 => (Letter::D, Accidental::Natural),
            3 => (Letter::D, Accidental::Sharp),
            4 => (Letter::E, Accidental::Natural),
            5 => (Letter::F, Accidental::Natural),
            6 => (Letter::F, Accidental::Sharp),
            7 => (Letter::G, Accidental::Natural),
            8 => (Letter::G, Accidental::Sharp),
            9 => (Letter::A, Accidental::Natural),
            10 => (Letter::A, Accidental::Sharp),
            11 => (Letter::B, Accidental::Natural),
            _ => unreachable!(),
        };
        Note { letter, accidental, octave, detune: 0.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Input {
    Knob(i8, i8, f64),
    Slider(i8, f64),
    Button(i8, i8, bool),

    Up(bool),
    Down(bool),
    Left(bool),
    Right(bool),

    Select(i8, bool),
}

pub struct Output {
    pub knobs: [Color; 24],
    pub buttons: [Color; 16],
    pub up: Color,
    pub down: Color,
    pub left: Color,
    pub right: Color,
    pub select: [bool; 4],
}

pub fn clear() -> Output {
    Output {
        knobs: [Color::Off; 24],
        buttons: [Color::Off; 16],
        up: Color::Off,
        down: Color::Off,
        left: Color::Off,
        right: Color::Off,
        select: [false; 4],
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    User,
    Factory
}

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Off,
    Red,
    Orange,
    Yellow,
    Green,
}

impl Color {
    pub fn mask(self) -> u8 {
        // bits:
        // 0..1: red brightness
        // 2..3: double buffering
        // 4..5: green brightness
        match self {
            Color::Off    => 0b001100,
            Color::Red    => 0b001101,
            Color::Orange => 0b111111,
            Color::Yellow => 0b111110,
            Color::Green  => 0b111100,
        }
    }
}

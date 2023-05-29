#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Color(f64, f64, f64);

impl Color {
    pub const fn off() -> Self     { Self(0.0, 0.0, 0.0) }
    pub const fn red() -> Self     { Self(0.0, 1.0, 0.0) }
    pub const fn green() -> Self   { Self(0.0, 0.0, 1.0) }
    pub const fn blue() -> Self    { Self(1.0, 0.0, 0.0) }
    pub const fn yellow() -> Self  { Self(0.0, 1.0, 1.0) }
    pub const fn cyan() -> Self    { Self(1.0, 1.0, 0.0) }
    pub const fn magenta() -> Self { Self(1.0, 0.0, 1.0) }
    pub const fn white() -> Self   { Self(1.0, 1.0, 1.0) }
}

impl From<Color> for (f64, f64, f64) {
    fn from(c: Color) -> Self {
        (c.0, c.1, c.2)
    }
}

pub type Input = ((i8, i8), f64);
pub type Output = [(f64, f64, f64); 81];

pub fn clear() -> Output {
    [(0.0, 0.0, 0.0); 81]
}

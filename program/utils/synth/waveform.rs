use std::sync::LazyLock;

use crate::utils::math::*;

const T_SAMPLES: usize = 1024 * 10;
const PHI_SAMPLES: usize = 128;

static SIN_TABLE: LazyLock<Wavetable> = LazyLock::new(|| {
    Wavetable::new_periodic(|t, _| (t * PI * 2.0).sin())
});

static TRI_TABLE: LazyLock<Wavetable> = LazyLock::new(|| {
    Wavetable::new_periodic(|t, _| (t * PI * 2.0).sin().asin() * 2.0 / PI)
});

static SAW_TABLE: LazyLock<Wavetable> = LazyLock::new(|| {
    Wavetable::new_periodic(|t, _| 2.0 * (t - t.floor()) - 1.0)
});

static SQUARE_TABLE: LazyLock<Wavetable> = LazyLock::new(|| {
    Wavetable::new_periodic(|t, _| if t - t.floor() < 0.5 { 1.0 } else { -1.0 })
});

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub enum Waveform {
    #[default]
    Sine,
    Tri,
    Saw,
    Square,
    Raw(Wavetable),
}

impl Waveform {
    pub fn sample(&self, t: f64) -> f64 {
        match self {
            Waveform::Sine => SIN_TABLE.sample(t),
            Waveform::Tri => TRI_TABLE.sample(t),
            Waveform::Saw => SAW_TABLE.sample(t),
            Waveform::Square => SQUARE_TABLE.sample(t),
            Waveform::Raw(table) => table.sample(t),
        }
    }

    pub fn sample_phi(&self, t: f64, phi: f64) -> f64 {
        match self {
            Waveform::Sine => SIN_TABLE.sample_phi(t, phi),
            Waveform::Tri => TRI_TABLE.sample_phi(t, phi),
            Waveform::Saw => SAW_TABLE.sample_phi(t, phi),
            Waveform::Square => SQUARE_TABLE.sample_phi(t, phi),
            Waveform::Raw(table) => table.sample_phi(t, phi),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Wavetable {
    data: Vec<Vec<f64>>,
}

impl Wavetable {
    pub fn new_periodic(f: impl Fn(f64, f64) -> f64) -> Self {
        let mut phis = Vec::with_capacity(PHI_SAMPLES);
        for i in 0..PHI_SAMPLES {
            let phi = i as f64 / PHI_SAMPLES as f64;

            let mut ts = Vec::with_capacity(T_SAMPLES);
            for j in 0..T_SAMPLES {
                let t = j as f64 / T_SAMPLES as f64;
                ts.push(f(t, phi));
            }

            phis.push(ts);
        }
        Self { data: phis }
    }

    pub fn sample(&self, t: f64) -> f64 {
        let n = T_SAMPLES as f64   - 1.0;

        // get indices in the t domain
        let f = (t * n) % n;
        let fr = f.fract();
        let t0 = f.floor() as usize;
        let t1 = f.ceil() as usize;

        // interpolate at phi=0
        let v0 = self.data[0][t0];
        let v1 = self.data[0][t1];
        fr.lerp(v0..v1)
    }

    pub fn sample_phi(&self, t: f64, phi: f64) -> f64 {
        let i_n = PHI_SAMPLES as f64 - 1.0;
        let j_n = T_SAMPLES as f64   - 1.0;

        // get indices in the phi domain
        let i_f = (phi * i_n) % i_n;
        let i_fr = i_f.fract();
        let phi0 = i_f.floor() as usize;
        let phi1 = i_f.ceil() as usize;

        // get indices in the t domain
        let j_f = (t * j_n) % j_n;
        let j_fr = j_f.fract();
        let t0 = j_f.floor() as usize;
        let t1 = j_f.ceil() as usize;

        // get the four values around t and phi
        let v00 = self.data[phi0][t0];
        let v01 = self.data[phi0][t1];
        let v10 = self.data[phi1][t0];
        let v11 = self.data[phi1][t1];

        // bilinear interpolation first along t, then along phi
        let v0 = j_fr.lerp(v00..v01);
        let v1 = j_fr.lerp(v10..v11);
        i_fr.lerp(v0..v1)
    }
}

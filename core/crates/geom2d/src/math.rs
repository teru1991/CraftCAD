#[derive(Clone, Copy, Debug)]
pub struct Epsilon {
    pub dist: f64,
    pub angle: f64,
    pub round: f64,
}

impl Default for Epsilon {
    fn default() -> Self {
        Self {
            dist: 1e-6,
            angle: 1e-6,
            round: 1e-6,
        }
    }
}

pub fn is_finite_pt(x: f64, y: f64) -> bool {
    x.is_finite() && y.is_finite()
}

pub fn round_step(v: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return v;
    }
    (v / step).round() * step
}

pub fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

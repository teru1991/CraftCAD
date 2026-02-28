#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeterminismConfig {
    pub seed: u64,
    pub epsilon: f64,
    pub rounding_decimals: u8,
}

impl DeterminismConfig {
    pub const fn new(seed: u64, epsilon: f64, rounding_decimals: u8) -> Self {
        Self {
            seed,
            epsilon,
            rounding_decimals,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn from_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        // 64-bit LCG (Numerical Recipes variant constants)
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
}

pub fn round_f64(value: f64, decimals: u8) -> f64 {
    let factor = 10f64.powi(i32::from(decimals));
    (value * factor).round() / factor
}

pub fn nearly_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() <= epsilon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = SeededRng::from_seed(42);
        let mut b = SeededRng::from_seed(42);

        let seq_a = [a.next_u64(), a.next_u64(), a.next_u64()];
        let seq_b = [b.next_u64(), b.next_u64(), b.next_u64()];

        assert_eq!(seq_a, seq_b);
    }

    #[test]
    fn rounding_and_epsilon_basics() {
        assert_eq!(round_f64(1.23456, 3), 1.235);
        assert!(nearly_equal(10.0, 10.0009, 0.001));
        assert!(!nearly_equal(10.0, 10.01, 0.001));
    }
}

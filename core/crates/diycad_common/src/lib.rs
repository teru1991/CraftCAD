pub mod determinism;

use thiserror::Error;

pub use determinism::{nearly_equal, round_f64, DeterminismConfig, SeededRng};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiycadError {
    #[error("invalid state: {0}")]
    InvalidState(String),
}

pub type DiycadResult<T> = Result<T, DiycadError>;

pub fn init_logging() {
    // logging backend will be introduced in a later bootstrap phase.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_message_is_stable() {
        let err = DiycadError::InvalidState("oops".into());
        assert_eq!(err.to_string(), "invalid state: oops");
    }
}

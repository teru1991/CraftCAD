pub mod determinism;
pub mod diagnostics;
pub mod logging;
pub mod paths;
pub mod settings;

use thiserror::Error;

pub use determinism::{nearly_equal, round_f64, DeterminismConfig, SeededRng};
pub use diagnostics::{collect_basic_diagnostics, BasicDiagnostics};
pub use logging::{init_logging, log_info};
pub use settings::{Settings, UiSettings};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiycadError {
    #[error("invalid state: {0}")]
    InvalidState(String),
}

pub type DiycadResult<T> = Result<T, DiycadError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_message_is_stable() {
        let err = DiycadError::InvalidState("oops".into());
        assert_eq!(err.to_string(), "invalid state: oops");
    }
}

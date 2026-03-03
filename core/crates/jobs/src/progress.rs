#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Progress reporting with monotonic guarantees.

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// Immutable snapshot for UI polling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    /// Fraction in [0.0, 1.0].
    pub fraction: f32,
    /// Optional total step count.
    pub total_steps: Option<u64>,
    /// Completed steps.
    pub completed_steps: u64,
    /// Short UI-safe status message.
    pub message: String,
}

/// Thread-safe reporter with monotonic update semantics.
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    inner: Arc<Mutex<ProgressSnapshot>>,
}

impl ProgressReporter {
    /// Create a fresh reporter.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressSnapshot {
                fraction: 0.0,
                total_steps: None,
                completed_steps: 0,
                message: String::new(),
            })),
        }
    }

    /// Current progress snapshot.
    pub fn snapshot(&self) -> ProgressSnapshot {
        self.inner.lock().expect("progress mutex poisoned").clone()
    }

    /// Set total steps; keeps fraction monotonic.
    pub fn set_total_steps(&self, n: u64) {
        let mut g = self.inner.lock().expect("progress mutex poisoned");
        g.total_steps = Some(n.max(1));
        let total = g.total_steps.expect("just set");
        let frac = (g.completed_steps as f32) / (total as f32);
        if frac > g.fraction {
            g.fraction = frac.min(1.0);
        }
    }

    /// Advance one step and update fraction monotonically.
    pub fn advance(&self) {
        let mut g = self.inner.lock().expect("progress mutex poisoned");
        g.completed_steps = g.completed_steps.saturating_add(1);
        if let Some(t) = g.total_steps {
            let frac = (g.completed_steps as f32) / (t as f32);
            if frac > g.fraction {
                g.fraction = frac.min(1.0);
            }
        }
    }

    /// Set explicit fraction; decreases are ignored.
    pub fn set_fraction(&self, fraction: f32) {
        let mut g = self.inner.lock().expect("progress mutex poisoned");
        let clamped = if fraction.is_nan() {
            0.0
        } else {
            fraction.clamp(0.0, 1.0)
        };
        if clamped > g.fraction {
            g.fraction = clamped;
        }
    }

    /// Set human-readable message (length capped).
    pub fn set_message(&self, message: impl Into<String>) {
        let mut g = self.inner.lock().expect("progress mutex poisoned");
        let mut s = message.into();
        if s.len() > 200 {
            s.truncate(200);
        }
        g.message = s;
    }

    /// Mark as fully completed.
    pub fn finish(&self) {
        let mut g = self.inner.lock().expect("progress mutex poisoned");
        g.fraction = 1.0;
        if let Some(total) = g.total_steps {
            g.completed_steps = total;
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

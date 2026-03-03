#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! LOD selection with fixed thresholds + hysteresis (deterministic).
//!
//! Policy: thresholds are constant rules (SSOT). Hysteresis avoids flicker near boundaries.

use serde::{Deserialize, Serialize};

/// LOD rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LodMode {
    /// Full quality rendering.
    Full,
    /// Medium quality rendering.
    Medium,
    /// Coarse rendering.
    Coarse,
}

/// LOD selection config.
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// Zoom threshold for Full->Medium transition.
    pub to_medium_zoom: f32,
    /// Zoom threshold for Medium->Coarse transition.
    pub to_coarse_zoom: f32,
    /// Hysteresis percentage around thresholds (0.0..0.5).
    pub hysteresis: f32,
    /// Complexity cutoff for Medium mode.
    pub max_complexity_medium: u32,
    /// Complexity cutoff for Coarse mode.
    pub max_complexity_coarse: u32,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            to_medium_zoom: 0.6,
            to_coarse_zoom: 0.25,
            hysteresis: 0.08,
            max_complexity_medium: 2000,
            max_complexity_coarse: 300,
        }
    }
}

/// Deterministic LOD selector.
pub struct LodSelector {
    cfg: LodConfig,
}

impl LodSelector {
    /// Create selector.
    pub fn new(cfg: LodConfig) -> Self {
        Self { cfg }
    }

    /// Select LOD mode with fixed thresholds and hysteresis.
    pub fn select(&self, zoom: f32, _viewport_px: (u32, u32), prev: Option<LodMode>) -> LodMode {
        let z = zoom.max(0.0);

        let med = self.cfg.to_medium_zoom;
        let crs = self.cfg.to_coarse_zoom;
        let h = self.cfg.hysteresis.clamp(0.0, 0.5);

        match prev.unwrap_or(LodMode::Full) {
            LodMode::Full => {
                if z <= med * (1.0 - h) {
                    LodMode::Medium
                } else {
                    LodMode::Full
                }
            }
            LodMode::Medium => {
                if z <= crs * (1.0 - h) {
                    LodMode::Coarse
                } else if z >= med * (1.0 + h) {
                    LodMode::Full
                } else {
                    LodMode::Medium
                }
            }
            LodMode::Coarse => {
                if z >= crs * (1.0 + h) {
                    LodMode::Medium
                } else {
                    LodMode::Coarse
                }
            }
        }
    }

    /// Decide if primitive should be kept in the selected mode.
    pub fn keep_primitive(&self, mode: &LodMode, p: &crate::Primitive) -> bool {
        match mode {
            LodMode::Full => true,
            LodMode::Medium => p.complexity <= self.cfg.max_complexity_medium,
            LodMode::Coarse => p.complexity <= self.cfg.max_complexity_coarse,
        }
    }
}

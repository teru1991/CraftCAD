use crate::model::Units;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_entities: usize,
    pub max_depth: usize,
}

impl Limits {
    pub fn default_for_tests() -> Self {
        Self {
            max_bytes: 4 * 1024 * 1024,
            max_entities: 50_000,
            max_depth: 128,
        }
    }
}

pub type IoLimits = Limits;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Determinism {
    pub seed: u64,
    pub round_step: f64,
    pub close_eps: f64,
    pub approx_eps: f64,
    pub approx_min_segments: usize,
    pub approx_max_segments: usize,
    pub approx_max_iter: usize,
    pub join_eps: f64,
    pub dedupe_eps: f64,
    pub tiny_segment_min_len: f64,
}

impl Determinism {
    pub fn default_for_tests() -> Self {
        Self {
            seed: 0,
            round_step: 1e-6,
            close_eps: 1e-6,
            approx_eps: 1e-3,
            approx_min_segments: 8,
            approx_max_segments: 128,
            approx_max_iter: 24,
            join_eps: 1e-6,
            dedupe_eps: 1e-6,
            tiny_segment_min_len: 1e-9,
        }
    }

    pub fn for_tests() -> Self {
        Self::default_for_tests()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OriginPolicy {
    Keep,
    MoveToZero,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub limits: Limits,
    pub determinism: Determinism,
    pub allow_unit_guess: bool,
    pub enable_postprocess: bool,
    pub enable_approx: bool,
}

impl ImportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            limits: Limits::default_for_tests(),
            determinism: Determinism::default_for_tests(),
            allow_unit_guess: true,
            enable_postprocess: true,
            enable_approx: true,
        }
    }

    pub fn determinism_tag(&self) -> String {
        format!(
            "seed={};round_step={:.12};close_eps={:.12};approx_eps={:.12}",
            self.determinism.seed,
            self.determinism.round_step,
            self.determinism.close_eps,
            self.determinism.approx_eps
        )
    }

    pub fn determinism_tag_compact(&self) -> String {
        self.determinism_tag()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub determinism: Determinism,
    pub target_units: Units,
    pub origin_policy: OriginPolicy,
    pub postprocess: bool,
    pub enable_postprocess: bool,
    pub enable_approx: bool,
}

impl ExportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            determinism: Determinism::default_for_tests(),
            target_units: Units::Mm,
            origin_policy: OriginPolicy::Keep,
            postprocess: true,
            enable_postprocess: true,
            enable_approx: true,
        }
    }

    pub fn determinism_tag(&self) -> String {
        format!(
            "seed={};round_step={:.12};close_eps={:.12};approx_eps={:.12}",
            self.determinism.seed,
            self.determinism.round_step,
            self.determinism.close_eps,
            self.determinism.approx_eps
        )
    }

    pub fn as_import_like(&self) -> ImportOptions {
        ImportOptions {
            limits: Limits {
                max_bytes: usize::MAX,
                max_entities: usize::MAX,
                max_depth: usize::MAX,
            },
            allow_unit_guess: false,
            determinism: self.determinism.clone(),
            enable_postprocess: self.enable_postprocess && self.postprocess,
            enable_approx: self.enable_approx,
        }
    }
}

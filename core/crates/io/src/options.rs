use crate::model::Units;

#[derive(Debug, Clone)]
pub struct IoLimits {
    pub max_bytes: usize,
    pub max_entities: usize,
    pub max_depth: usize,
}

impl IoLimits {
    pub fn for_tests() -> Self {
        Self {
            max_bytes: 1024 * 1024,
            max_entities: 100_000,
            max_depth: 128,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Determinism {
    pub round_step: f64,
    pub close_eps: f64,
    pub seed: u64,
}

impl Determinism {
    pub fn for_tests() -> Self {
        Self {
            round_step: 1e-6,
            close_eps: 1e-6,
            seed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub limits: IoLimits,
    pub allow_unit_guess: bool,
    pub determinism: Determinism,
}

impl ImportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            limits: IoLimits::for_tests(),
            allow_unit_guess: true,
            determinism: Determinism::for_tests(),
        }
    }

    pub fn determinism_tag(&self) -> String {
        format!(
            "seed={};round_step={:.12};close_eps={:.12}",
            self.determinism.seed, self.determinism.round_step, self.determinism.close_eps
        )
    }
}

#[derive(Debug, Clone)]
pub enum OriginPolicy {
    Keep,
    MoveToZero,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub target_units: Units,
    pub origin_policy: OriginPolicy,
    pub postprocess: bool,
    pub determinism: Determinism,
}

impl ExportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            target_units: Units::Mm,
            origin_policy: OriginPolicy::Keep,
            postprocess: true,
            determinism: Determinism::for_tests(),
        }
    }

    pub fn determinism_tag(&self) -> String {
        format!(
            "seed={};round_step={:.12};close_eps={:.12}",
            self.determinism.seed, self.determinism.round_step, self.determinism.close_eps
        )
    }

    pub fn as_import_like(&self) -> ImportOptions {
        ImportOptions {
            limits: IoLimits {
                max_bytes: usize::MAX,
                max_entities: usize::MAX,
                max_depth: usize::MAX,
            },
            allow_unit_guess: false,
            determinism: self.determinism.clone(),
        }
    }
}

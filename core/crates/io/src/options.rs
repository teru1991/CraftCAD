use crate::model;

#[derive(Clone, Debug)]
pub struct ImportOptions {
    pub max_bytes: usize,
    pub max_entities: usize,
    pub max_nesting_depth: usize,
    pub allow_unit_guess: bool,
    pub approx_epsilon: f64,
    pub seed: u64,
}

#[derive(Clone, Debug)]
pub struct ExportOptions {
    pub target_units: model::Units,
    pub origin_policy: OriginPolicy,
    pub postprocess: bool,
    pub approx_epsilon: f64,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OriginPolicy {
    Keep,
    MoveToZero,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            max_bytes: 8 * 1024 * 1024,
            max_entities: 100_000,
            max_nesting_depth: 64,
            allow_unit_guess: true,
            approx_epsilon: 0.01,
            seed: 0,
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            target_units: model::Units::Mm,
            origin_policy: OriginPolicy::Keep,
            postprocess: false,
            approx_epsilon: 0.01,
            seed: 0,
        }
    }
}

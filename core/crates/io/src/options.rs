use crate::model::Units;

/// 入力制限（limits SSOT から注入される想定）
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

/// 決定性パラメータ（determinism SSOT から注入される想定）
#[derive(Debug, Clone)]
pub struct Determinism {
    pub round_step: f64,
    pub close_eps: f64,
    pub seed: u64,

    // approx/postprocess（PR3で追加）
    pub approx_eps: f64,
    pub approx_min_segments: usize,
    pub approx_max_segments: usize,
    pub approx_max_iter: usize,

    pub join_eps: f64,
    pub dedupe_eps: f64,
    pub tiny_segment_min_len: f64,
}

impl Determinism {
    pub fn for_tests() -> Self {
        Self {
            round_step: 1e-6,
            close_eps: 1e-6,
            seed: 0,

            approx_eps: 1e-3,
            approx_min_segments: 8,
            approx_max_segments: 1024,
            approx_max_iter: 24,

            join_eps: 1e-6,
            dedupe_eps: 1e-6,
            tiny_segment_min_len: 1e-9,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub limits: IoLimits,
    pub allow_unit_guess: bool,
    pub determinism: Determinism,

    // 呼び側が “曲線近似を適用するか” を決めるためのフラグ（support_matrix参照は呼び側責務）
    pub enable_approx: bool,
}

impl ImportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            limits: IoLimits::for_tests(),
            allow_unit_guess: true,
            determinism: Determinism::for_tests(),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    // export前の近似も呼び側が制御できる（support_matrix参照は呼び側責務）
    pub enable_approx: bool,
}

impl ExportOptions {
    pub fn default_for_tests() -> Self {
        Self {
            target_units: Units::Mm,
            origin_policy: OriginPolicy::Keep,
            postprocess: true,
            determinism: Determinism::for_tests(),
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

    /// normalize が ImportOptions を取る設計なので、export側パラメータを import-like に変換して渡す
    pub fn as_import_like(&self) -> ImportOptions {
        ImportOptions {
            limits: IoLimits {
                max_bytes: usize::MAX,
                max_entities: usize::MAX,
                max_depth: usize::MAX,
            },
            allow_unit_guess: false,
            determinism: self.determinism.clone(),
            enable_approx: self.enable_approx,
        }
    }
}

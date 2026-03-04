pub mod diag_codes {
    pub const DIAG_LOG_TRUNCATED: &str = "DIAG_LOG_TRUNCATED";
    pub const DIAG_PARAMS_TRUNCATED: &str = "DIAG_PARAMS_TRUNCATED";
    pub const DIAG_INPUTS_TRUNCATED: &str = "DIAG_INPUTS_TRUNCATED";
    pub const DIAG_STEP_TRUNCATED: &str = "DIAG_STEP_TRUNCATED";
    pub const DIAG_INTERNAL_ERROR: &str = "DIAG_INTERNAL_ERROR";
    pub const DIAG_SSOT_FINGERPRINT_PARTIAL: &str = "DIAG_SSOT_FINGERPRINT_PARTIAL";
}

#[derive(
    Clone, Copy, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Error,
}

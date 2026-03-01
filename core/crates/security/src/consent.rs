use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsentState {
    pub telemetry_opt_in: bool,
    pub support_zip_include_project: bool,
    pub remember_choice: bool,
}

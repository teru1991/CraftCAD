use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::reasons::{SecCode, SecError, SecResult, SecWarning};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentDecision {
    UseSaved,
    OneTime {
        telemetry_opt_in: bool,
        diagnostics_include_project: bool,
        diagnostics_include_inputs_copy: bool,
    },
    Cancel,
}

#[derive(Debug, Clone)]
pub struct ConsentLoadOutcome {
    pub state: ConsentState,
    pub warnings: Vec<SecWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsentState {
    pub version: u64,
    pub telemetry_opt_in: bool,
    pub diagnostics_include_project: bool,
    pub diagnostics_include_inputs_copy: bool,
    pub remember_choice: bool,
}

impl Default for ConsentState {
    fn default() -> Self {
        Self {
            version: 1,
            telemetry_opt_in: false,
            diagnostics_include_project: false,
            diagnostics_include_inputs_copy: false,
            remember_choice: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsentStore {
    path: PathBuf,
}

impl ConsentStore {
    pub fn new_default() -> SecResult<Self> {
        let dir = if let Ok(v) = std::env::var("CRAFTCAD_CONFIG_DIR") {
            PathBuf::from(v)
        } else {
            let base = dirs_next::config_dir().ok_or_else(|| {
                SecError::new(SecCode::SecSsotNotFound, "config_dir not available")
            })?;
            base.join("CraftCAD")
        };
        Ok(Self {
            path: dir.join("consent.json"),
        })
    }

    pub fn with_path(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> ConsentLoadOutcome {
        let mut warnings = Vec::new();
        let s = match std::fs::read_to_string(&self.path) {
            Ok(x) => x,
            Err(_) => {
                return ConsentLoadOutcome {
                    state: ConsentState::default(),
                    warnings,
                };
            }
        };
        match serde_json::from_str::<ConsentState>(&s) {
            Ok(st) => {
                if st.version < 1 {
                    warnings.push(SecWarning {
                        code: SecCode::SecConsentReset,
                        message: "consent version invalid; reset to defaults".into(),
                    });
                    return ConsentLoadOutcome {
                        state: ConsentState::default(),
                        warnings,
                    };
                }
                ConsentLoadOutcome {
                    state: st,
                    warnings,
                }
            }
            Err(_) => {
                warnings.push(SecWarning {
                    code: SecCode::SecConsentReset,
                    message: "consent file corrupted; reset to defaults".into(),
                });
                ConsentLoadOutcome {
                    state: ConsentState::default(),
                    warnings,
                }
            }
        }
    }

    pub fn save(&self, state: &ConsentState) -> SecResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SecError::new(
                    SecCode::SecSsotInvalid,
                    format!("create config dir failed: {e}"),
                )
            })?;
        }
        let s = serde_json::to_string_pretty(state).map_err(|e| {
            SecError::new(
                SecCode::SecSsotInvalid,
                format!("serialize consent failed: {e}"),
            )
        })?;
        // atomic write best-effort
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, s.as_bytes()).map_err(|e| {
            SecError::new(
                SecCode::SecSsotInvalid,
                format!("write temp consent failed: {e}"),
            )
        })?;
        std::fs::rename(&tmp, &self.path).map_err(|e| {
            SecError::new(
                SecCode::SecSsotInvalid,
                format!("rename consent failed: {e}"),
            )
        })?;
        Ok(())
    }
}

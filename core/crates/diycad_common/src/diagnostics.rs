use crate::paths::settings_path;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicDiagnostics {
    pub os: String,
    pub arch: String,
    pub app_version: String,
    pub settings_path: Option<String>,
    pub time: String,
}

pub fn collect_basic_diagnostics() -> BasicDiagnostics {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    BasicDiagnostics {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        settings_path: settings_path().map(|p| p.to_string_lossy().to_string()),
        time,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_contains_core_fields() {
        let diagnostics = collect_basic_diagnostics();
        assert!(!diagnostics.os.is_empty());
        assert!(!diagnostics.arch.is_empty());
        assert!(!diagnostics.app_version.is_empty());
        assert!(!diagnostics.time.is_empty());
    }
}

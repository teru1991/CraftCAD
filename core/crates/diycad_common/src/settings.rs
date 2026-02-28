use crate::paths::settings_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub recent_files: Vec<String>,
    pub ui: UiSettings,
    pub units: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            ui: UiSettings {
                theme: "system".to_string(),
                language: "ja-JP".to_string(),
            },
            units: "mm".to_string(),
        }
    }
}

impl Settings {
    // NOTE: This is app-level configuration and is stored separately from `.diycad` project files.
    pub fn load_or_default() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };

        let Ok(content) = fs::read_to_string(path) else {
            return Self::default();
        };

        serde_json::from_str::<Self>(&content).unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = settings_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "cannot resolve app data directory for settings",
            )
        })?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serde_roundtrip() {
        let original = Settings {
            recent_files: vec!["/tmp/a.diycad".to_string(), "/tmp/b.diycad".to_string()],
            ui: UiSettings {
                theme: "dark".to_string(),
                language: "en-US".to_string(),
            },
            units: "mm".to_string(),
        };

        let json = serde_json::to_string(&original).expect("serialize should succeed");
        let restored: Settings = serde_json::from_str(&json).expect("deserialize should succeed");

        assert_eq!(original, restored);
    }
}

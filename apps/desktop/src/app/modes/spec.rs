use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModePolicySsot {
    pub version: u32,
    pub modes: Vec<String>,
    pub keys: BTreeMap<String, String>,
    pub global_guards: Vec<ModeGuard>,
    pub transitions: Vec<ModeTransition>,
    pub dialogs: ModeDialogs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeGuard {
    pub id: String,
    pub condition: BTreeMap<String, serde_yaml::Value>,
    pub denies: Vec<ModeDeny>,
    pub required_recovery_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeDeny {
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    pub from: String,
    pub event: String,
    pub to: String,
    pub side_effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeDialogs {
    pub confirm_on_discard_dirty: bool,
}

#[derive(Debug, Clone)]
pub struct SsotPaths {
    pub mode_policy_md: PathBuf,
}

impl Default for SsotPaths {
    fn default() -> Self {
        Self {
            mode_policy_md: PathBuf::from("docs/specs/ux/mode_policy.md"),
        }
    }
}

fn extract_ssot_yaml(md: &str) -> Result<String, String> {
    let begin = "<!-- SSOT:BEGIN -->";
    let end = "<!-- SSOT:END -->";
    let b = md
        .find(begin)
        .ok_or_else(|| "missing SSOT:BEGIN".to_string())?;
    let e = md.find(end).ok_or_else(|| "missing SSOT:END".to_string())?;
    if e <= b {
        return Err("SSOT markers are in wrong order".to_string());
    }
    Ok(md[b + begin.len()..e].trim().to_string())
}

pub fn load_mode_policy(paths: &SsotPaths) -> Result<ModePolicySsot, String> {
    let md = fs::read_to_string(&paths.mode_policy_md)
        .map_err(|e| format!("failed to read {}: {}", paths.mode_policy_md.display(), e))?;
    let yaml = extract_ssot_yaml(&md)?;
    let v: serde_yaml::Value =
        serde_yaml::from_str(&yaml).map_err(|e| format!("yaml parse error: {}", e))?;
    let mut m = match v {
        serde_yaml::Value::Mapping(map) => map,
        _ => return Err("SSOT yaml must be mapping".to_string()),
    };
    m.remove(serde_yaml::Value::String("kind".to_string()));
    let cleaned = serde_yaml::Value::Mapping(m);
    let pol: ModePolicySsot =
        serde_yaml::from_value(cleaned).map_err(|e| format!("SSOT decode error: {}", e))?;
    validate_mode_policy(&pol)?;
    Ok(pol)
}

pub fn validate_mode_policy(pol: &ModePolicySsot) -> Result<(), String> {
    if pol.version != 1 {
        return Err(format!("unsupported mode_policy version: {}", pol.version));
    }
    if pol.modes.is_empty() {
        return Err("modes empty".to_string());
    }
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for t in &pol.transitions {
        if t.from.trim().is_empty() || t.event.trim().is_empty() || t.to.trim().is_empty() {
            return Err("transition has empty field".to_string());
        }
        let k = (t.from.clone(), t.event.clone());
        if !seen.insert(k.clone()) {
            return Err(format!("duplicate transition: from={} event={}", k.0, k.1));
        }
    }
    Ok(())
}

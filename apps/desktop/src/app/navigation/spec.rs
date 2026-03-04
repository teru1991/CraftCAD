use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationPolicySsot {
    pub version: u32,
    pub breadcrumbs: BreadcrumbsPolicy,
    pub backstack: BackstackPolicy,
    pub deep_links: DeepLinkPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbsPolicy {
    pub pattern: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackstackPolicy {
    pub enabled: bool,
    pub max_depth: u32,
    pub esc_does_not_pop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkPolicy {
    pub targets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SsotPaths {
    pub navigation_policy_md: PathBuf,
}

impl Default for SsotPaths {
    fn default() -> Self {
        Self {
            navigation_policy_md: PathBuf::from("docs/specs/ux/navigation_policy.md"),
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

pub fn load_navigation_policy(paths: &SsotPaths) -> Result<NavigationPolicySsot, String> {
    let md = fs::read_to_string(&paths.navigation_policy_md).map_err(|e| {
        format!(
            "failed to read {}: {}",
            paths.navigation_policy_md.display(),
            e
        )
    })?;
    let yaml = extract_ssot_yaml(&md)?;
    let v: serde_yaml::Value =
        serde_yaml::from_str(&yaml).map_err(|e| format!("yaml parse error: {}", e))?;
    let mut m = match v {
        serde_yaml::Value::Mapping(map) => map,
        _ => return Err("SSOT yaml must be mapping".to_string()),
    };
    m.remove(serde_yaml::Value::String("kind".to_string()));
    let cleaned = serde_yaml::Value::Mapping(m);
    let pol: NavigationPolicySsot =
        serde_yaml::from_value(cleaned).map_err(|e| format!("SSOT decode error: {}", e))?;
    validate_navigation_policy(&pol)?;
    Ok(pol)
}

pub fn validate_navigation_policy(pol: &NavigationPolicySsot) -> Result<(), String> {
    if pol.version != 1 {
        return Err(format!(
            "unsupported navigation_policy version: {}",
            pol.version
        ));
    }
    if pol.breadcrumbs.pattern.is_empty() {
        return Err("breadcrumbs.pattern empty".to_string());
    }
    if pol.backstack.max_depth == 0 || pol.backstack.max_depth > 500 {
        return Err("backstack.max_depth out of range".to_string());
    }
    Ok(())
}

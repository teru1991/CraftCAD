use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingFlowSsot {
    pub version: u32,
    pub flow_id: String,
    pub entrypoints: Vec<String>,
    pub completion: Completion,
    pub steps: Vec<StepSpec>,
    pub policy: Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub all_of: Vec<CompletionReq>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionReq {
    Op {
        op: String,
        args: Option<std::collections::BTreeMap<String, serde_yaml::Value>>,
    },
    Job {
        job: String,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepSpec {
    pub id: String,
    pub title_key: String,
    pub body_key: String,
    pub required: RequiredExpr,
    pub next: String,
    pub can_skip: bool,
    pub links: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredExpr {
    pub any_of: Vec<ReqAtom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReqAtom {
    Op {
        op: String,
        args: Option<std::collections::BTreeMap<String, serde_yaml::Value>>,
    },
    Job {
        job: String,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub allow_rerun: bool,
    pub allow_skip: bool,
    pub sample_open_mode: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SsotPaths {
    pub onboarding_flow_md: PathBuf,
}

impl Default for SsotPaths {
    fn default() -> Self {
        Self {
            onboarding_flow_md: PathBuf::from("docs/specs/ux/onboarding_flow.md"),
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

pub fn load_onboarding_flow(paths: &SsotPaths) -> Result<OnboardingFlowSsot, String> {
    let md = fs::read_to_string(&paths.onboarding_flow_md).map_err(|e| {
        format!(
            "failed to read {}: {}",
            paths.onboarding_flow_md.display(),
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
    let flow: OnboardingFlowSsot =
        serde_yaml::from_value(cleaned).map_err(|e| format!("SSOT decode error: {}", e))?;

    validate_flow(&flow)?;
    Ok(flow)
}

pub fn validate_flow(flow: &OnboardingFlowSsot) -> Result<(), String> {
    if flow.version != 1 {
        return Err(format!(
            "unsupported onboarding_flow version: {}",
            flow.version
        ));
    }
    if flow.flow_id.trim().is_empty() {
        return Err("flow_id empty".to_string());
    }
    if flow.steps.is_empty() {
        return Err("steps empty".to_string());
    }

    let mut ids = std::collections::BTreeSet::new();
    for s in &flow.steps {
        if s.id.trim().is_empty() {
            return Err("step id empty".to_string());
        }
        if !ids.insert(s.id.clone()) {
            return Err(format!("duplicate step id: {}", s.id));
        }
        if s.title_key.trim().is_empty() || s.body_key.trim().is_empty() {
            return Err(format!("step {} missing i18n key(s)", s.id));
        }
        if s.required.any_of.is_empty() {
            return Err(format!("step {} required.any_of empty", s.id));
        }
    }
    for s in &flow.steps {
        if s.next != "done" && !ids.contains(&s.next) {
            return Err(format!("step {} has invalid next: {}", s.id, s.next));
        }
    }

    let mut cur = flow.steps[0].id.clone();
    let mut visited = std::collections::BTreeSet::new();
    loop {
        if !visited.insert(cur.clone()) {
            return Err(format!("cycle detected at {}", cur));
        }
        let s = flow
            .steps
            .iter()
            .find(|x| x.id == cur)
            .ok_or_else(|| "internal missing step".to_string())?;
        if s.next == "done" {
            break;
        }
        cur = s.next.clone();
    }
    Ok(())
}

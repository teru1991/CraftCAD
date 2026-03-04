use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum UxSsotDoc {
    #[serde(rename = "onboarding_flow")]
    OnboardingFlow(OnboardingFlow),
    #[serde(rename = "error_ux_policy")]
    ErrorUxPolicy(ErrorUxPolicy),
    #[serde(rename = "mode_policy")]
    ModePolicy(ModePolicy),
    #[serde(rename = "navigation_policy")]
    NavigationPolicy(NavigationPolicy),
    #[serde(rename = "sample_library")]
    SampleLibrary(SampleLibrary),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMeta {
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingFlow {
    pub version: u32,
    pub flow_id: String,
    pub entrypoints: Vec<String>,
    pub completion: Completion,
    pub steps: Vec<OnboardingStep>,
    pub policy: OnboardingPolicy,
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
        args: Option<BTreeMap<String, serde_yaml::Value>>,
    },
    Job {
        job: String,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub id: String,
    pub title_key: String,
    pub body_key: String,
    pub required: RequirementExpr,
    pub next: String,
    pub can_skip: bool,
    pub links: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementExpr {
    pub any_of: Vec<RequirementAtom>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequirementAtom {
    Op {
        op: String,
        args: Option<BTreeMap<String, serde_yaml::Value>>,
    },
    Job {
        job: String,
        status: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPolicy {
    pub allow_rerun: bool,
    pub allow_skip: bool,
    pub sample_open_mode: String,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorUxPolicy {
    pub version: u32,
    pub ui_contract: ErrorUiContract,
    pub required_actions: Vec<String>,
    pub mapping_contract: MappingContract,
    pub logging: ErrorLogging,
    pub safety: ErrorSafety,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorUiContract {
    pub order: Vec<String>,
    pub actions_max: u32,
    pub detail_collapsed_by_default: bool,
    pub pii_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingContract {
    pub require_title_key: bool,
    pub require_detail_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLogging {
    pub record_on_show: bool,
    pub record_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSafety {
    pub redaction_required: bool,
    pub consent_required_for_supportzip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModePolicy {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationPolicy {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleLibrary {
    pub version: u32,
    pub samples: Vec<SampleMeta>,
    pub policy: SamplePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMeta {
    pub id: String,
    pub title_key: String,
    pub description_key: String,
    pub file: String,
    pub read_only: bool,
    pub tags: Vec<String>,
    pub limits: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplePolicy {
    pub must_be_openable_offline: bool,
    pub schema_compat: BTreeMap<String, String>,
    pub update_rules: BTreeMap<String, serde_yaml::Value>,
}

pub fn known_error_actions() -> BTreeSet<String> {
    [
        "OpenDocs",
        "OpenSettings",
        "CreateSupportZip",
        "RunMigrateTool",
        "RetryLastJob",
        "JumpToEntity",
        "DuplicateSampleAsProject",
        "CancelActiveJob",
        "ShowJobProgress",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect()
}

pub fn ensure_unique_transitions(policy: &ModePolicy) -> Result<(), String> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for t in &policy.transitions {
        let key = (t.from.clone(), t.event.clone());
        if !seen.insert(key.clone()) {
            return Err(format!(
                "duplicate transition for (from,event)=({},{})",
                key.0, key.1
            ));
        }
    }
    Ok(())
}

pub fn ensure_steps_chain(flow: &OnboardingFlow) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for s in &flow.steps {
        if !ids.insert(s.id.clone()) {
            return Err(format!("duplicate onboarding step id: {}", s.id));
        }
    }
    for s in &flow.steps {
        if s.next != "done" && !ids.contains(&s.next) {
            return Err(format!(
                "onboarding step {} has invalid next: {}",
                s.id, s.next
            ));
        }
    }
    if flow.steps.is_empty() {
        return Err("onboarding steps empty".to_string());
    }

    let mut cur = flow.steps[0].id.clone();
    let mut visited = BTreeSet::new();
    loop {
        if !visited.insert(cur.clone()) {
            return Err(format!("onboarding flow has cycle at {}", cur));
        }
        let step = flow
            .steps
            .iter()
            .find(|s| s.id == cur)
            .expect("step id must exist");
        if step.next == "done" {
            break;
        }
        cur = step.next.clone();
    }
    Ok(())
}

pub fn ensure_actions_subset(policy: &ErrorUxPolicy) -> Result<(), String> {
    let known = known_error_actions();
    for a in &policy.required_actions {
        if !known.contains(a) {
            return Err(format!("unknown required action in error_ux_policy: {}", a));
        }
    }
    Ok(())
}

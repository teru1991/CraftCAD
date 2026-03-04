use super::reason_catalog::ReasonCatalog;
use super::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ErrorMapper {
    pub actions_max: usize,
}

impl Default for ErrorMapper {
    fn default() -> Self {
        Self { actions_max: 3 }
    }
}

impl ErrorMapper {
    pub fn map(&self, app_err: &AppError, catalog: &ReasonCatalog) -> UserFacingError {
        let ent = catalog.get(&app_err.reason_code);

        let title_key = ent
            .and_then(|e| e.title_key.clone())
            .unwrap_or_else(|| catalog.fallback_title_key().to_string());
        let detail_key = ent
            .and_then(|e| e.detail_key.clone())
            .unwrap_or_else(|| catalog.fallback_detail_key().to_string());
        let why_key = ent
            .and_then(|e| e.why_key.clone())
            .or_else(|| Some(catalog.fallback_why_key().to_string()));

        let mut actions: Vec<UserFacingAction> = vec![];
        if let Some(e) = ent {
            for a in &e.actions {
                if actions.len() >= self.actions_max {
                    break;
                }
                if let Some(k) = parse_action_kind(&a.kind) {
                    let label_key = a
                        .label_key
                        .clone()
                        .unwrap_or_else(|| default_action_label_key(&k).to_string());
                    let args = json_args_to_string_map(&a.args);
                    actions.push(UserFacingAction {
                        kind: k,
                        label_key,
                        args,
                    });
                }
            }
        }

        let doc_link = ent.and_then(|e| e.doc_link.clone());
        let debug_ref = DebugRef {
            reason_code: app_err.reason_code.clone(),
            severity: app_err.severity.clone(),
            job_id: app_err.job_id.clone(),
            op_id: app_err.op_id.clone(),
            context: app_err.context.clone(),
        };

        let display_hash = compute_hash(
            &title_key,
            &detail_key,
            &why_key,
            &actions,
            &doc_link,
            &debug_ref,
        );

        UserFacingError {
            title_key,
            detail_key,
            why_key,
            actions,
            doc_link,
            debug_ref,
            display_hash,
        }
    }
}

fn parse_action_kind(s: &str) -> Option<ActionKind> {
    match s {
        "OpenDocs" => Some(ActionKind::OpenDocs),
        "OpenSettings" => Some(ActionKind::OpenSettings),
        "CreateSupportZip" => Some(ActionKind::CreateSupportZip),
        "RunMigrateTool" => Some(ActionKind::RunMigrateTool),
        "RetryLastJob" => Some(ActionKind::RetryLastJob),
        "JumpToEntity" => Some(ActionKind::JumpToEntity),
        "DuplicateSampleAsProject" => Some(ActionKind::DuplicateSampleAsProject),
        "CancelActiveJob" => Some(ActionKind::CancelActiveJob),
        "ShowJobProgress" => Some(ActionKind::ShowJobProgress),
        _ => None,
    }
}

fn default_action_label_key(k: &ActionKind) -> &'static str {
    match k {
        ActionKind::OpenDocs => "ux.error.action.open_docs",
        ActionKind::OpenSettings => "ux.error.action.open_settings",
        ActionKind::CreateSupportZip => "ux.error.action.create_support_zip",
        ActionKind::RunMigrateTool => "ux.error.action.run_migrate",
        ActionKind::RetryLastJob => "ux.error.action.retry",
        ActionKind::JumpToEntity => "ux.error.action.jump",
        ActionKind::DuplicateSampleAsProject => "ux.error.action.duplicate_sample",
        ActionKind::CancelActiveJob => "ux.error.action.cancel_job",
        ActionKind::ShowJobProgress => "ux.error.action.show_progress",
    }
}

fn json_args_to_string_map(
    args: &std::collections::BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (k, v) in args {
        let s = match v {
            serde_json::Value::String(x) => x.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => v.to_string(),
        };
        out.insert(k.clone(), s);
    }
    out
}

fn compute_hash(
    title_key: &str,
    detail_key: &str,
    why_key: &Option<String>,
    actions: &[UserFacingAction],
    doc_link: &Option<String>,
    debug_ref: &DebugRef,
) -> String {
    #[derive(serde::Serialize)]
    struct Canon<'a> {
        title_key: &'a str,
        detail_key: &'a str,
        why_key: &'a Option<String>,
        actions: &'a [UserFacingAction],
        doc_link: &'a Option<String>,
        debug_ref: &'a DebugRef,
    }
    let c = Canon {
        title_key,
        detail_key,
        why_key,
        actions,
        doc_link,
        debug_ref,
    };
    let bytes = serde_json::to_vec(&c).expect("canon json");
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

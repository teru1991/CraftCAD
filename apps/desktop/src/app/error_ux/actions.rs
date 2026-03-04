use super::*;

#[derive(Debug, Clone)]
pub enum ActionEffect {
    OpenDocs { link: String },
    OpenSettings { section: String },
    RequestConsentForSupportZip,
    CreateSupportZip,
    RunMigrateTool { input: Option<String> },
    RetryLastJob,
    JumpToEntity { entity_id: String },
    DuplicateSampleAsProject { sample_id: String },
    CancelActiveJob,
    ShowJobProgress,
    None,
}

#[derive(Debug, Clone)]
pub enum ActionExecError {
    MissingArg(String),
    Denied(String),
    Failed(String),
}

pub struct ActionExecutor;

impl ActionExecutor {
    pub fn exec(action: &UserFacingAction) -> Result<ActionEffect, ActionExecError> {
        let a = action;
        match a.kind {
            ActionKind::OpenDocs => {
                let link = a
                    .args
                    .get("doc_link")
                    .cloned()
                    .unwrap_or_else(|| "ux/error".to_string());
                Ok(ActionEffect::OpenDocs { link })
            }
            ActionKind::OpenSettings => {
                let section = a
                    .args
                    .get("section")
                    .cloned()
                    .unwrap_or_else(|| "general".to_string());
                Ok(ActionEffect::OpenSettings { section })
            }
            ActionKind::CreateSupportZip => Ok(ActionEffect::RequestConsentForSupportZip),
            ActionKind::RunMigrateTool => {
                let input = a.args.get("input").cloned();
                Ok(ActionEffect::RunMigrateTool { input })
            }
            ActionKind::RetryLastJob => Ok(ActionEffect::RetryLastJob),
            ActionKind::JumpToEntity => {
                let Some(id) = a.args.get("entity_id").cloned() else {
                    return Err(ActionExecError::MissingArg("entity_id".to_string()));
                };
                Ok(ActionEffect::JumpToEntity { entity_id: id })
            }
            ActionKind::DuplicateSampleAsProject => {
                let Some(id) = a.args.get("sample_id").cloned() else {
                    return Err(ActionExecError::MissingArg("sample_id".to_string()));
                };
                Ok(ActionEffect::DuplicateSampleAsProject { sample_id: id })
            }
            ActionKind::CancelActiveJob => Ok(ActionEffect::CancelActiveJob),
            ActionKind::ShowJobProgress => Ok(ActionEffect::ShowJobProgress),
        }
    }

    pub fn to_app_error(err: &ActionExecError, action: &UserFacingAction) -> AppError {
        let mut ctx = std::collections::BTreeMap::new();
        ctx.insert("action".to_string(), action.kind.as_str().to_string());
        match err {
            ActionExecError::MissingArg(a) => {
                ctx.insert("missing".to_string(), a.clone());
                AppError {
                    reason_code: "UI_ACTION_FAILED_MISSING_ARG".to_string(),
                    severity: Severity::Error,
                    context: ctx,
                    job_id: None,
                    op_id: None,
                }
            }
            ActionExecError::Denied(msg) => {
                ctx.insert("denied".to_string(), msg.clone());
                AppError {
                    reason_code: "UI_ACTION_FAILED_DENIED".to_string(),
                    severity: Severity::Error,
                    context: ctx,
                    job_id: None,
                    op_id: None,
                }
            }
            ActionExecError::Failed(msg) => {
                ctx.insert("failed".to_string(), msg.clone());
                AppError {
                    reason_code: "UI_ACTION_FAILED".to_string(),
                    severity: Severity::Error,
                    context: ctx,
                    job_id: None,
                    op_id: None,
                }
            }
        }
    }
}

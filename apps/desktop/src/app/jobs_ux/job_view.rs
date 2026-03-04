use super::adapters::{JobKind, JobSnapshot, JobState};

#[derive(Debug, Clone)]
pub struct JobUxView {
    pub visible: bool,
    pub job_id: Option<String>,
    pub kind: Option<JobKind>,
    pub title_key: String,
    pub progress01: Option<f32>,
    pub stage_key: Option<String>,
    pub can_cancel: bool,
    pub is_running: bool,
    pub is_done: bool,
    pub last_error_reason_code: Option<String>,
}

impl JobUxView {
    pub fn idle() -> Self {
        Self {
            visible: false,
            job_id: None,
            kind: None,
            title_key: "ux.job.idle".to_string(),
            progress01: None,
            stage_key: None,
            can_cancel: false,
            is_running: false,
            is_done: false,
            last_error_reason_code: None,
        }
    }

    pub fn from_snapshot(s: &JobSnapshot, visible: bool) -> Self {
        let (progress01, stage_key, can_cancel, is_running, is_done, last_err) = match &s.state {
            JobState::Queued => (
                Some(0.0),
                Some("ux.job.stage.queued".to_string()),
                true,
                true,
                false,
                None,
            ),
            JobState::Running { progress01, stage } => (
                Some(*progress01),
                stage.clone().map(|x| format!("ux.job.stage.{}", x)),
                true,
                true,
                false,
                None,
            ),
            JobState::Succeeded { .. } => (
                Some(1.0),
                Some("ux.job.stage.succeeded".to_string()),
                false,
                false,
                true,
                None,
            ),
            JobState::Failed { reason_code, .. } => (
                None,
                Some("ux.job.stage.failed".to_string()),
                false,
                false,
                true,
                Some(reason_code.clone()),
            ),
            JobState::Cancelled => (
                None,
                Some("ux.job.stage.cancelled".to_string()),
                false,
                false,
                true,
                None,
            ),
        };
        let title_key = match s.kind {
            JobKind::NestJob => "ux.job.title.nest",
            JobKind::ExportJob => "ux.job.title.export",
            JobKind::OpenProject => "ux.job.title.open",
            JobKind::SaveProject => "ux.job.title.save",
            JobKind::Import => "ux.job.title.import",
        }
        .to_string();

        Self {
            visible,
            job_id: Some(s.job_id.clone()),
            kind: Some(s.kind.clone()),
            title_key,
            progress01,
            stage_key,
            can_cancel,
            is_running,
            is_done,
            last_error_reason_code: last_err,
        }
    }
}

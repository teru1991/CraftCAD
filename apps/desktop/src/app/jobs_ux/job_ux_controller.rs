use super::adapters::*;

#[derive(Debug, Clone)]
pub enum JobUxEvent {
    Enqueue(EnqueueRequest),
    CancelActive,
    RetryLastFailed,
    HidePanel,
    ShowPanel,
}

#[derive(Debug, Clone)]
pub enum JobUxEffect {
    None,
    ShowError {
        reason_code: String,
        job_id: Option<String>,
        context: std::collections::BTreeMap<String, String>,
    },
    NotifyOnboardingJobSucceeded {
        kind: JobKind,
        job_id: String,
    },
    NotifyModesJobRunning {
        running: bool,
    },
}

pub struct JobUxController {
    visible: bool,
    active_job_id: Option<String>,
    last_enqueued_req: Option<EnqueueRequest>,
    last_failed_job: Option<(
        EnqueueRequest,
        String,
        String,
        std::collections::BTreeMap<String, String>,
    )>,
}

impl JobUxController {
    pub fn new() -> Self {
        Self {
            visible: false,
            active_job_id: None,
            last_enqueued_req: None,
            last_failed_job: None,
        }
    }

    pub fn view(&self, q: &dyn IJobQueue) -> super::job_view::JobUxView {
        let Some(id) = &self.active_job_id else {
            return super::job_view::JobUxView::idle();
        };
        if let Some(s) = q.snapshot(id) {
            super::job_view::JobUxView::from_snapshot(&s, self.visible)
        } else {
            super::job_view::JobUxView::idle()
        }
    }

    pub fn handle_event(
        &mut self,
        q: &mut dyn IJobQueue,
        ev: JobUxEvent,
    ) -> Result<Vec<JobUxEffect>, String> {
        match ev {
            JobUxEvent::ShowPanel => {
                self.visible = true;
                Ok(vec![JobUxEffect::None])
            }
            JobUxEvent::HidePanel => {
                self.visible = false;
                Ok(vec![JobUxEffect::None])
            }
            JobUxEvent::Enqueue(req) => {
                let resp = q.enqueue(req.clone())?;
                self.last_enqueued_req = Some(req);
                self.active_job_id = Some(resp.job_id.clone());
                self.visible = true;
                Ok(vec![JobUxEffect::NotifyModesJobRunning { running: true }])
            }
            JobUxEvent::CancelActive => {
                let Some(id) = &self.active_job_id else {
                    return Ok(vec![JobUxEffect::None]);
                };
                q.cancel(id)?;
                Ok(vec![JobUxEffect::None])
            }
            JobUxEvent::RetryLastFailed => {
                let Some((req, _, _, _)) = self.last_failed_job.clone() else {
                    return Ok(vec![JobUxEffect::None]);
                };
                let resp = q.enqueue(req.clone())?;
                self.last_enqueued_req = Some(req);
                self.active_job_id = Some(resp.job_id.clone());
                self.visible = true;
                Ok(vec![JobUxEffect::NotifyModesJobRunning { running: true }])
            }
        }
    }

    pub fn tick(&mut self, q: &dyn IJobQueue) -> Vec<JobUxEffect> {
        let mut out = vec![];
        let Some(id) = &self.active_job_id else {
            return out;
        };
        let Some(s) = q.snapshot(id) else {
            return out;
        };

        match &s.state {
            JobState::Queued | JobState::Running { .. } => {
                out.push(JobUxEffect::NotifyModesJobRunning { running: true });
            }
            JobState::Succeeded { .. } => {
                out.push(JobUxEffect::NotifyModesJobRunning { running: false });
                out.push(JobUxEffect::NotifyOnboardingJobSucceeded {
                    kind: s.kind.clone(),
                    job_id: s.job_id.clone(),
                });
                self.active_job_id = None;
            }
            JobState::Cancelled => {
                out.push(JobUxEffect::NotifyModesJobRunning { running: false });
                self.active_job_id = None;
            }
            JobState::Failed {
                reason_code,
                context,
            } => {
                out.push(JobUxEffect::NotifyModesJobRunning { running: false });
                out.push(JobUxEffect::ShowError {
                    reason_code: reason_code.clone(),
                    job_id: Some(s.job_id.clone()),
                    context: context.clone(),
                });
                if let Some(req) = self.last_enqueued_req.clone() {
                    self.set_last_failed(
                        req,
                        s.job_id.clone(),
                        reason_code.clone(),
                        context.clone(),
                    );
                }
                self.active_job_id = None;
            }
        }
        out
    }

    pub fn set_last_failed(
        &mut self,
        req: EnqueueRequest,
        job_id: String,
        reason_code: String,
        context: std::collections::BTreeMap<String, String>,
    ) {
        self.last_failed_job = Some((req, job_id, reason_code, context));
    }
}

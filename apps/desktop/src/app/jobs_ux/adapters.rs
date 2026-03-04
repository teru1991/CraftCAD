use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobKind {
    NestJob,
    ExportJob,
    OpenProject,
    SaveProject,
    Import,
}

impl JobKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobKind::NestJob => "NestJob",
            JobKind::ExportJob => "ExportJob",
            JobKind::OpenProject => "OpenProject",
            JobKind::SaveProject => "SaveProject",
            JobKind::Import => "Import",
        }
    }
}

#[derive(Debug, Clone)]
pub enum JobState {
    Queued,
    Running {
        progress01: f32,
        stage: Option<String>,
    },
    Succeeded {
        output: BTreeMap<String, String>,
    },
    Failed {
        reason_code: String,
        context: BTreeMap<String, String>,
    },
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct JobSnapshot {
    pub job_id: String,
    pub kind: JobKind,
    pub state: JobState,
    pub created_unix_ms: i64,
}

#[derive(Debug, Clone)]
pub struct EnqueueRequest {
    pub kind: JobKind,
    pub args: BTreeMap<String, String>,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct EnqueueResponse {
    pub job_id: String,
}

pub trait IJobQueue {
    fn enqueue(&mut self, req: EnqueueRequest) -> Result<EnqueueResponse, String>;
    fn cancel(&mut self, job_id: &str) -> Result<(), String>;
    fn snapshot(&self, job_id: &str) -> Option<JobSnapshot>;
    fn active_jobs(&self) -> Vec<JobSnapshot>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryJobQueue {
    next_id: u64,
    items: BTreeMap<String, JobSnapshot>,
}

impl InMemoryJobQueue {
    pub fn mark_running(&mut self, job_id: &str, progress01: f32, stage: Option<String>) {
        if let Some(s) = self.items.get_mut(job_id) {
            s.state = JobState::Running { progress01, stage };
        }
    }

    pub fn mark_succeeded(&mut self, job_id: &str, output: BTreeMap<String, String>) {
        if let Some(s) = self.items.get_mut(job_id) {
            s.state = JobState::Succeeded { output };
        }
    }

    pub fn mark_failed(
        &mut self,
        job_id: &str,
        reason_code: String,
        context: BTreeMap<String, String>,
    ) {
        if let Some(s) = self.items.get_mut(job_id) {
            s.state = JobState::Failed {
                reason_code,
                context,
            };
        }
    }
}

impl IJobQueue for InMemoryJobQueue {
    fn enqueue(&mut self, req: EnqueueRequest) -> Result<EnqueueResponse, String> {
        self.next_id += 1;
        let job_id = format!("job-{}", self.next_id);
        self.items.insert(
            job_id.clone(),
            JobSnapshot {
                job_id: job_id.clone(),
                kind: req.kind,
                state: JobState::Queued,
                created_unix_ms: 0,
            },
        );
        Ok(EnqueueResponse { job_id })
    }

    fn cancel(&mut self, job_id: &str) -> Result<(), String> {
        match self.items.get_mut(job_id) {
            Some(s) => {
                s.state = JobState::Cancelled;
                Ok(())
            }
            None => Err(format!("job not found: {}", job_id)),
        }
    }

    fn snapshot(&self, job_id: &str) -> Option<JobSnapshot> {
        self.items.get(job_id).cloned()
    }

    fn active_jobs(&self) -> Vec<JobSnapshot> {
        self.items.values().cloned().collect()
    }
}

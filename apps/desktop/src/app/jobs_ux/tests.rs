use super::*;
use adapters::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct FakeQueue {
    next_id: u32,
    snaps: std::collections::BTreeMap<String, JobSnapshot>,
}

impl IJobQueue for FakeQueue {
    fn enqueue(&mut self, req: EnqueueRequest) -> Result<EnqueueResponse, String> {
        self.next_id += 1;
        let id = format!("j{}", self.next_id);
        let snap = JobSnapshot {
            job_id: id.clone(),
            kind: req.kind,
            state: JobState::Queued,
            created_unix_ms: 0,
        };
        self.snaps.insert(id.clone(), snap);
        Ok(EnqueueResponse { job_id: id })
    }

    fn cancel(&mut self, job_id: &str) -> Result<(), String> {
        if let Some(s) = self.snaps.get_mut(job_id) {
            s.state = JobState::Cancelled;
        }
        Ok(())
    }

    fn snapshot(&self, job_id: &str) -> Option<JobSnapshot> {
        self.snaps.get(job_id).cloned()
    }

    fn active_jobs(&self) -> Vec<JobSnapshot> {
        self.snaps.values().cloned().collect()
    }
}

#[test]
fn enqueue_sets_job_running_true() {
    let mut q = FakeQueue::default();
    let mut c = job_ux_controller::JobUxController::new();
    let req = EnqueueRequest {
        kind: JobKind::NestJob,
        args: BTreeMap::new(),
        priority: 0,
    };
    let eff = c
        .handle_event(&mut q, job_ux_controller::JobUxEvent::Enqueue(req))
        .unwrap();
    assert!(eff.iter().any(|e| matches!(
        e,
        job_ux_controller::JobUxEffect::NotifyModesJobRunning { running: true }
    )));
}

#[test]
fn cancel_transitions_to_not_running_on_tick() {
    let mut q = FakeQueue::default();
    let mut c = job_ux_controller::JobUxController::new();
    let req = EnqueueRequest {
        kind: JobKind::NestJob,
        args: BTreeMap::new(),
        priority: 0,
    };
    let _ = c
        .handle_event(&mut q, job_ux_controller::JobUxEvent::Enqueue(req))
        .unwrap();
    let id = c.view(&q).job_id.unwrap();
    q.cancel(&id).unwrap();
    let effs = c.tick(&q);
    assert!(effs.iter().any(|e| matches!(
        e,
        job_ux_controller::JobUxEffect::NotifyModesJobRunning { running: false }
    )));
}

#[test]
fn failed_job_routes_error_and_retry_reenqueues() {
    let mut q = FakeQueue::default();
    let mut c = job_ux_controller::JobUxController::new();
    let req = EnqueueRequest {
        kind: JobKind::ExportJob,
        args: BTreeMap::from([("preset".to_string(), "pdf".to_string())]),
        priority: 1,
    };
    let _ = c
        .handle_event(&mut q, job_ux_controller::JobUxEvent::Enqueue(req))
        .unwrap();
    let id = c.view(&q).job_id.unwrap();
    if let Some(s) = q.snaps.get_mut(&id) {
        s.state = JobState::Failed {
            reason_code: "EXPORT_PDF_FAILED".to_string(),
            context: BTreeMap::from([("target".to_string(), "pdf".to_string())]),
        };
    }
    let effs = c.tick(&q);
    assert!(effs.iter().any(|e| matches!(
        e,
        job_ux_controller::JobUxEffect::ShowError { reason_code, .. } if reason_code == "EXPORT_PDF_FAILED"
    )));

    let retry = c
        .handle_event(&mut q, job_ux_controller::JobUxEvent::RetryLastFailed)
        .unwrap();
    assert!(retry.iter().any(|e| matches!(
        e,
        job_ux_controller::JobUxEffect::NotifyModesJobRunning { running: true }
    )));
}

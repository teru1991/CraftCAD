#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use craftcad_jobs::{
    CancelToken, Job, JobCancelledOrError, JobCoreError, JobKind, JobPriority, JobQueue,
    JobQueueConfig, JobReason, ProgressReporter,
};

struct SleepJob {
    id: String,
    prio: JobPriority,
    ms: u64,
    started: Arc<Mutex<Vec<String>>>,
}

impl Job for SleepJob {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn kind(&self) -> JobKind {
        JobKind::Other
    }

    fn priority(&self) -> JobPriority {
        self.prio
    }

    fn run(
        &mut self,
        cancel: CancelToken,
        progress: ProgressReporter,
    ) -> Result<serde_json::Value, JobCancelledOrError> {
        self.started
            .lock()
            .expect("started mutex poisoned")
            .push(self.id.clone());
        progress.set_total_steps(10);
        for _ in 0..10 {
            cancel.check()?;
            std::thread::sleep(Duration::from_millis(self.ms / 10));
            progress.advance();
        }
        Ok(serde_json::json!({"id": self.id}))
    }
}

struct PanicJob;

impl Job for PanicJob {
    fn id(&self) -> String {
        "PANIC".to_string()
    }

    fn kind(&self) -> JobKind {
        JobKind::Other
    }

    fn priority(&self) -> JobPriority {
        JobPriority::High
    }

    fn run(
        &mut self,
        _cancel: CancelToken,
        _progress: ProgressReporter,
    ) -> Result<serde_json::Value, JobCancelledOrError> {
        panic!("boom")
    }
}

#[test]
fn high_priority_runs_first_and_fifo_within_priority() {
    let q = JobQueue::new(JobQueueConfig { max_queue_len: 64 }).expect("queue create");
    let started = Arc::new(Mutex::new(Vec::<String>::new()));

    let _n1 = q
        .submit(SleepJob {
            id: "N1".into(),
            prio: JobPriority::Normal,
            ms: 20,
            started: Arc::clone(&started),
        })
        .expect("submit N1");
    let _h1 = q
        .submit(SleepJob {
            id: "H1".into(),
            prio: JobPriority::High,
            ms: 20,
            started: Arc::clone(&started),
        })
        .expect("submit H1");
    let _n2 = q
        .submit(SleepJob {
            id: "N2".into(),
            prio: JobPriority::Normal,
            ms: 20,
            started: Arc::clone(&started),
        })
        .expect("submit N2");
    let _h2 = q
        .submit(SleepJob {
            id: "H2".into(),
            prio: JobPriority::High,
            ms: 20,
            started: Arc::clone(&started),
        })
        .expect("submit H2");
    let _h3 = q
        .submit(SleepJob {
            id: "H3".into(),
            prio: JobPriority::High,
            ms: 20,
            started: Arc::clone(&started),
        })
        .expect("submit H3");

    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if started.lock().expect("started mutex poisoned").len() >= 5 {
            break;
        }
        assert!(Instant::now() < deadline, "timeout");
        std::thread::sleep(Duration::from_millis(10));
    }

    let s = started.lock().expect("started mutex poisoned").clone();

    let first_norm = s
        .iter()
        .position(|x| x.starts_with('N'))
        .expect("normal jobs started");
    let last_high = s
        .iter()
        .rposition(|x| x.starts_with('H'))
        .expect("high jobs started");
    assert!(
        last_high < first_norm,
        "High jobs must start before any Normal: {:?}",
        s
    );

    let h = s
        .iter()
        .filter(|x| x.starts_with('H'))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(h, vec!["H1", "H2", "H3"], "High FIFO violated: {:?}", h);

    let n = s
        .iter()
        .filter(|x| x.starts_with('N'))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(n, vec!["N1", "N2"], "Normal FIFO violated: {:?}", n);
}

#[test]
fn cancel_stops_job_and_reports_cancelled() {
    let q = JobQueue::new(JobQueueConfig { max_queue_len: 64 }).expect("queue create");
    let started = Arc::new(Mutex::new(Vec::<String>::new()));

    let h = q
        .submit(SleepJob {
            id: "C1".into(),
            prio: JobPriority::High,
            ms: 200,
            started: Arc::clone(&started),
        })
        .expect("submit C1");

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if started
            .lock()
            .expect("started mutex poisoned")
            .iter()
            .any(|x| x == "C1")
        {
            break;
        }
        assert!(Instant::now() < deadline, "timeout waiting start");
        std::thread::sleep(Duration::from_millis(5));
    }

    h.cancel();

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(r) = h.try_result() {
            assert!(!r.ok);
            assert_eq!(r.reason, Some(JobReason::JobCancelled));
            break;
        }
        assert!(Instant::now() < deadline, "timeout waiting result");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn progress_is_monotonic() {
    let q = JobQueue::new(JobQueueConfig { max_queue_len: 64 }).expect("queue create");
    let started = Arc::new(Mutex::new(Vec::<String>::new()));

    let h = q
        .submit(SleepJob {
            id: "P1".into(),
            prio: JobPriority::High,
            ms: 50,
            started: Arc::clone(&started),
        })
        .expect("submit P1");

    let mut last = 0.0f32;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let p = h.progress();
        assert!(
            p.fraction + 1e-6 >= last,
            "progress went backwards: {} -> {}",
            last,
            p.fraction
        );
        last = p.fraction;
        if let Some(r) = h.try_result() {
            assert!(r.ok);
            break;
        }
        assert!(Instant::now() < deadline, "timeout");
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn queue_full_is_rejected() {
    let q = JobQueue::new(JobQueueConfig { max_queue_len: 1 }).expect("queue create");
    let started = Arc::new(Mutex::new(Vec::<String>::new()));

    let _h1 = q
        .submit(SleepJob {
            id: "Q1".into(),
            prio: JobPriority::Low,
            ms: 200,
            started: Arc::clone(&started),
        })
        .expect("submit Q1");
    let e = q
        .submit(SleepJob {
            id: "Q2".into(),
            prio: JobPriority::Low,
            ms: 200,
            started,
        })
        .expect_err("Q2 should be rejected");
    match e {
        JobCoreError::QueueFull => {}
        other => panic!("expected QueueFull, got: {other:?}"),
    }
}

#[test]
fn panics_are_caught_and_reported() {
    let q = JobQueue::new(JobQueueConfig { max_queue_len: 8 }).expect("queue create");
    let h = q.submit(PanicJob).expect("submit panic job");

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(r) = h.try_result() {
            assert!(!r.ok);
            assert_eq!(r.reason, Some(JobReason::JobPanicked));
            break;
        }
        assert!(Instant::now() < deadline, "timeout waiting panic result");
        std::thread::sleep(Duration::from_millis(5));
    }
}

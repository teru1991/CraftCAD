#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Priority job queue with single worker (deterministic FIFO per priority).

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::cancel::CancelToken;
use crate::job::{Job, JobCancelledOrError, JobPriority, JobResult};
use crate::progress::{ProgressReporter, ProgressSnapshot};
use crate::reasons::JobReason;
use crate::{JobCoreError, JobCoreResult};

/// Queue runtime config.
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    /// Hard cap on queued jobs (DoS safety).
    pub max_queue_len: usize,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_len: 1024,
        }
    }
}

/// Handle returned to caller for poll/cancel.
#[derive(Debug, Clone)]
pub struct JobHandle {
    /// Stable id copied from the submitted job.
    pub id: String,
    cancel: CancelToken,
    progress: ProgressReporter,
    result: Arc<Mutex<Option<JobResult>>>,
}

impl JobHandle {
    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Read current progress snapshot.
    pub fn progress(&self) -> ProgressSnapshot {
        self.progress.snapshot()
    }

    /// Try getting final result (non-blocking).
    pub fn try_result(&self) -> Option<JobResult> {
        self.result.lock().expect("result mutex poisoned").clone()
    }
}

struct QueuedJob {
    priority: JobPriority,
    submit_seq: u64,
    job: Box<dyn Job>,
    handle: JobHandle,
}

impl PartialEq for QueuedJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.submit_seq == other.submit_seq
    }
}

impl Eq for QueuedJob {}

impl PartialOrd for QueuedJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedJob {
    fn cmp(&self, other: &Self) -> Ordering {
        let rank = |p: JobPriority| -> u8 {
            match p {
                JobPriority::High => 3,
                JobPriority::Normal => 2,
                JobPriority::Low => 1,
            }
        };
        rank(self.priority)
            .cmp(&rank(other.priority))
            .then_with(|| other.submit_seq.cmp(&self.submit_seq))
    }
}

struct Inner {
    heap: BinaryHeap<QueuedJob>,
    submit_seq: u64,
    shutdown: bool,
}

/// Deterministic single-worker job queue.
pub struct JobQueue {
    cfg: JobQueueConfig,
    state: Arc<(Mutex<Inner>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}

impl JobQueue {
    /// Create queue and start worker thread.
    pub fn new(cfg: JobQueueConfig) -> JobCoreResult<Self> {
        if cfg.max_queue_len == 0 {
            return Err(JobCoreError::InvalidConfig(
                "max_queue_len must be > 0".to_string(),
            ));
        }

        let state = Arc::new((
            Mutex::new(Inner {
                heap: BinaryHeap::new(),
                submit_seq: 0,
                shutdown: false,
            }),
            Condvar::new(),
        ));

        let worker_state = Arc::clone(&state);
        let worker = thread::Builder::new()
            .name("craftcad-job-worker".to_string())
            .spawn(move || worker_loop(worker_state))
            .map_err(|e| JobCoreError::InvalidConfig(format!("failed to spawn worker: {e}")))?;

        Ok(Self {
            cfg,
            state,
            worker: Some(worker),
        })
    }

    /// Submit a job.
    pub fn submit<J: Job>(&self, job: J) -> JobCoreResult<JobHandle> {
        let (lock, cv) = &*self.state;
        let mut g = lock.lock().expect("queue mutex poisoned");
        if g.shutdown {
            return Err(JobCoreError::Shutdown);
        }
        if g.heap.len() >= self.cfg.max_queue_len {
            return Err(JobCoreError::QueueFull);
        }

        let id = job.id();
        let cancel = CancelToken::new();
        let progress = ProgressReporter::new();
        let result = Arc::new(Mutex::new(None));

        let handle = JobHandle {
            id: id.clone(),
            cancel: cancel.clone(),
            progress: progress.clone(),
            result: Arc::clone(&result),
        };

        let submit_seq = g.submit_seq;
        g.submit_seq = g.submit_seq.wrapping_add(1);
        g.heap.push(QueuedJob {
            priority: job.priority(),
            submit_seq,
            job: Box::new(job),
            handle: handle.clone(),
        });
        cv.notify_one();

        Ok(handle)
    }

    /// Shutdown and join worker thread.
    pub fn shutdown(mut self) {
        let (lock, cv) = &*self.state;
        {
            let mut g = lock.lock().expect("queue mutex poisoned");
            g.shutdown = true;
        }
        cv.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for JobQueue {
    fn drop(&mut self) {
        let (lock, cv) = &*self.state;
        {
            let mut g = lock.lock().expect("queue mutex poisoned");
            g.shutdown = true;
        }
        cv.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn worker_loop(state: Arc<(Mutex<Inner>, Condvar)>) {
    let (lock, cv) = &*state;

    loop {
        let mut g = lock.lock().expect("queue mutex poisoned");
        while g.heap.is_empty() && !g.shutdown {
            g = cv.wait(g).expect("queue cv wait failed");
        }

        if g.shutdown {
            while let Some(q) = g.heap.pop() {
                let mut r = q.handle.result.lock().expect("result mutex poisoned");
                *r = Some(JobResult::failed(
                    JobReason::JobQueueShutdown,
                    "queue shutdown",
                ));
            }
            break;
        }

        let mut q = match g.heap.pop() {
            Some(x) => x,
            None => continue,
        };
        drop(g);

        let cancel = q.handle.cancel.clone();
        let progress = q.handle.progress.clone();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            q.job.run(cancel.clone(), progress.clone())
        }));

        let result = match outcome {
            Ok(Ok(output)) => {
                progress.finish();
                JobResult::success(output)
            }
            Ok(Err(JobCancelledOrError::Cancelled)) => JobResult::cancelled(),
            Ok(Err(JobCancelledOrError::Failed(message))) => {
                JobResult::failed(JobReason::JobFailed, message)
            }
            Err(_) => JobResult::failed(JobReason::JobPanicked, "job panicked (caught)"),
        };

        let mut slot = q.handle.result.lock().expect("result mutex poisoned");
        *slot = Some(result);
    }
}

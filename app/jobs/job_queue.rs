use crate::jobs::cancel_token::CancelToken;
use crate::jobs::job_types::{JobPriority, JobType};
use std::collections::VecDeque;

pub struct JobHandle {
    pub job_type: JobType,
    pub progress: f32,
    pub cancel_token: CancelToken,
}

pub struct JobQueue {
    q: VecDeque<(JobPriority, JobHandle)>,
}

impl JobQueue {
    pub fn new() -> Self {
        Self { q: VecDeque::new() }
    }

    pub fn enqueue(&mut self, priority: JobPriority, handle: JobHandle) {
        self.q.push_back((priority, handle));
        self.q.make_contiguous().sort_by_key(|(p, _)| *p);
    }

    pub fn pop_next(&mut self) -> Option<JobHandle> {
        self.q.pop_front().map(|(_, j)| j)
    }
}

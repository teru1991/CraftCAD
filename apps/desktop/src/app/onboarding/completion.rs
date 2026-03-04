use super::spec::{OnboardingFlowSsot, ReqAtom};
use super::tutorial_state::{TutorialModel, TutorialState};

#[derive(Debug, Clone)]
pub enum JobStatus {
    Succeeded,
    Failed,
    Running,
    Cancelled,
    Unknown,
}

pub trait OpLogQuery {
    fn has_op(
        &self,
        op_kind: &str,
        args: &std::collections::BTreeMap<String, serde_yaml::Value>,
    ) -> bool;
}

pub trait JobQuery {
    fn job_status(&self, job_kind: &str) -> JobStatus;
}

pub struct CompletionEngine {
    flow: OnboardingFlowSsot,
}

impl CompletionEngine {
    pub fn new(flow: OnboardingFlowSsot) -> Self {
        Self { flow }
    }

    pub fn flow(&self) -> &OnboardingFlowSsot {
        &self.flow
    }

    pub fn tick(
        &self,
        model: &TutorialModel,
        state: &mut TutorialState,
        oplog: &dyn OpLogQuery,
        jobs: &dyn JobQuery,
        now_unix_ms: i64,
    ) {
        if state.started_unix_ms.is_none() {
            state.mark_started(now_unix_ms);
        }
        if state.is_done {
            return;
        }

        let cur_id = state.current_step.0.clone();
        let Some(step_spec) = self.flow.steps.iter().find(|s| s.id == cur_id) else {
            return;
        };
        if self.eval_required(&step_spec.required.any_of, oplog, jobs) {
            state.complete_current_and_advance(model);
        }

        if self.eval_completion(oplog, jobs) {
            state.mark_finished(now_unix_ms);
        }
    }

    fn eval_required(
        &self,
        any_of: &[ReqAtom],
        oplog: &dyn OpLogQuery,
        jobs: &dyn JobQuery,
    ) -> bool {
        for atom in any_of {
            match atom {
                ReqAtom::Op { op, args } => {
                    let a = args.clone().unwrap_or_default();
                    if oplog.has_op(op, &a) {
                        return true;
                    }
                }
                ReqAtom::Job { job, status } => {
                    let st = jobs.job_status(job);
                    if status_matches(&st, status) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn eval_completion(&self, oplog: &dyn OpLogQuery, jobs: &dyn JobQuery) -> bool {
        for r in &self.flow.completion.all_of {
            match r {
                super::spec::CompletionReq::Op { op, args } => {
                    let a = args.clone().unwrap_or_default();
                    if !oplog.has_op(op, &a) {
                        return false;
                    }
                }
                super::spec::CompletionReq::Job { job, status } => {
                    let st = jobs.job_status(job);
                    if !status_matches(&st, status) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

fn status_matches(actual: &JobStatus, expected: &str) -> bool {
    matches!(
        (actual, expected),
        (JobStatus::Succeeded, "Succeeded")
            | (JobStatus::Failed, "Failed")
            | (JobStatus::Running, "Running")
            | (JobStatus::Cancelled, "Cancelled")
    )
}

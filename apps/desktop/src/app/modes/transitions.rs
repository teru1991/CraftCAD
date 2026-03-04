#[derive(Debug, Clone, Default)]
pub struct ModeState {
    pub job_running: bool,
}

pub fn sync_job_running_from_jobs_ux(state: &mut ModeState, running: bool) {
    state.job_running = running;
}

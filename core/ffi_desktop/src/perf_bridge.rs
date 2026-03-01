// IMPORTANT (CraftCAD Product Quality Rules):
// - UI must never block on long tasks: job queue + cancel + progress are mandatory.

use craftcad_perf::{PerfReport, PerfSession};

pub fn begin_perf_session(dataset_id: &str, schema_version: &str, seed: u64) -> PerfSession {
    PerfSession::start(dataset_id)
        .tag_schema_version(schema_version.to_string())
        .tag_seed(seed)
}

pub fn end_perf_session(session: PerfSession) -> PerfReport {
    session.finish()
}

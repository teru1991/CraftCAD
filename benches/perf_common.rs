#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use craftcad_perf::{check_report_against_budgets, load_budgets, PerfReport, PerfSession};
use serde_json::json;

pub const DETERMINISM_TAG: &str = "seed=0;eps=1e-6;round=1e-4";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root from core/crates/perf")
        .to_path_buf()
}

pub fn load_dataset_json(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(repo_root().join(path)).expect("read dataset");
    serde_json::from_str(&text).expect("parse dataset json")
}

pub fn artifacts_dir() -> PathBuf {
    let p = repo_root().join("benches").join("artifacts");
    std::fs::create_dir_all(&p).expect("create benches/artifacts");
    p
}

pub fn write_report(name: &str, report: &PerfReport) {
    let budgets = load_budgets(repo_root().join("docs/specs/perf/budgets.json")).expect("load budgets");
    let violations = check_report_against_budgets(report, &budgets);

    let out = json!({
        "dataset_id": report.dataset_id,
        "determinism_tag": DETERMINISM_TAG,
        "report": report,
        "budget_warnings": violations,
    });

    let dir = artifacts_dir();
    let path = dir.join(format!("{name}.json"));
    let text = serde_json::to_string_pretty(&out).expect("serialize report");
    std::fs::write(&path, text).expect("write report");
    println!("[bench] wrote report: {}", path.display());
}

pub fn session(dataset_id: &str) -> PerfSession {
    PerfSession::start(dataset_id)
        .tag_seed(0)
        .tag_schema_version("1")
}

use std::path::{Path, PathBuf};

use craftcad_perf::{check_report_against_budgets, load_budgets, perf_span, PerfSession};
use serde_json::json;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("repo root from core/crates/perf")
        .to_path_buf()
}

#[test]
fn perf_smoke_generates_report_and_checks_budgets() {
    let root = repo_root();

    let budgets = load_budgets(root.join("docs/specs/perf/budgets.json")).expect("load budgets SSOT");

    let dataset_path = root.join("tests/datasets/heavy_sample_v1/heavy_sample_v1.json");
    let dataset_text = std::fs::read_to_string(&dataset_path).expect("read dataset");
    let dataset_json: serde_json::Value = serde_json::from_str(&dataset_text).expect("parse dataset JSON");

    let session = PerfSession::start("heavy_sample_v1")
        .tag_schema_version("1")
        .tag_seed(0);

    perf_span!("open", {
        perf_span!("diycad.open.preflight", {
            let _ = dataset_json
                .get("schema_version")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
        });

        perf_span!("diycad.open.salvage.parts", {
            let mut acc: u64 = 0;
            if let Some(ents) = dataset_json.get("entities").and_then(|v| v.as_array()) {
                for e in ents {
                    acc = acc.wrapping_add(e.get("id").and_then(|v| v.as_u64()).unwrap_or(0));
                }
            }
            std::hint::black_box(acc);
        });
    });

    perf_span!("render.frame", {
        perf_span!("render.frame.build_primitives", {
            let mut sum: u64 = 0;
            if let Some(ents) = dataset_json.get("entities").and_then(|v| v.as_array()) {
                for e in ents {
                    sum = sum.wrapping_add(e.get("id").and_then(|v| v.as_u64()).unwrap_or(0));
                }
            }
            std::hint::black_box(sum);
        });
        perf_span!("render.frame.draw", {});
    });

    perf_span!("io.import.total", {});
    perf_span!("io.export.total", {});
    perf_span!("diycad.save.total", {});

    let report = session.finish();

    let violations = check_report_against_budgets(&report, &budgets);
    let warnings: Vec<serde_json::Value> = violations
        .iter()
        .map(|e| {
            json!({
                "reason_code": e.code,
                "severity": format!("{:?}", e.severity),
                "message": e.message,
            })
        })
        .collect();

    let out_dir = root.join("tests/perf/artifacts");
    std::fs::create_dir_all(&out_dir).expect("create artifacts dir");
    let out_path = out_dir.join("perf_report_heavy_sample_v1.json");
    let artifact = json!({
        "report": report,
        "budget_warnings": warnings,
        "dataset_path": dataset_path.strip_prefix(&root).unwrap_or(Path::new("tests/datasets/heavy_sample_v1/heavy_sample_v1.json")),
    });
    let text = serde_json::to_string_pretty(&artifact).expect("serialize report artifact");
    std::fs::write(&out_path, text).expect("write report");

    assert_eq!(artifact["report"]["dataset_id"], "heavy_sample_v1");
    assert!(out_path.exists(), "perf artifact must exist");
}

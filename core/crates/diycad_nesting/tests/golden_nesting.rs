use craftcad_serialize::{Document, NestJob, PartPlacementStatusKind};
use diycad_geom::EpsilonPolicy;
use diycad_nesting::{run_nesting, RunLimits};

fn load(name: &str) -> serde_json::Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../tests/golden/nesting")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(p).expect("fixture")).expect("json")
}

fn run_case(doc_name: &str, job_name: &str, expected_name: &str) {
    let doc: Document = serde_json::from_value(load(doc_name)).expect("doc parse");
    let job: NestJob = serde_json::from_value(load(job_name)).expect("job parse");
    let expected = load(expected_name);

    let (result, trace) = run_nesting(
        &job,
        &doc,
        &EpsilonPolicy::default(),
        RunLimits {
            iteration_limit: 20,
            time_limit_ms: 500,
        },
    )
    .unwrap_or_else(|e| {
        panic!(
            "nesting failed code={} seed={} eps={:?} doc={} job={}",
            e.code,
            job.seed,
            EpsilonPolicy::default(),
            doc_name,
            job_name
        )
    });

    let placed = result
        .per_part_status
        .iter()
        .filter(|s| matches!(s.status, PartPlacementStatusKind::Placed))
        .count();
    let unplaced = result
        .per_part_status
        .iter()
        .filter(|s| matches!(s.status, PartPlacementStatusKind::Unplaced))
        .count();

    assert_eq!(
        trace.stop_reason,
        expected["stop_reason"].as_str().unwrap(),
        "doc={doc_name}"
    );
    assert_eq!(
        result.metrics.sheet_count_used,
        expected["sheet_count_used"].as_u64().unwrap() as u32,
        "doc={doc_name}"
    );
    assert_eq!(
        placed,
        expected["placed_count"].as_u64().unwrap() as usize,
        "doc={doc_name}"
    );
    assert_eq!(
        unplaced,
        expected["unplaced_count"].as_u64().unwrap() as usize,
        "doc={doc_name}"
    );

    if let Some(rc) = expected.get("reason_code") {
        let rcode = result
            .per_part_status
            .iter()
            .find_map(|s| s.reason.as_ref().map(|r| r.code.clone()))
            .unwrap_or_default();
        assert_eq!(rcode, rc.as_str().unwrap(), "doc={doc_name}");
    }
}

#[test]
fn golden_nesting_small() {
    run_case("doc_small.json", "job_small.json", "expected_small.json");
}

#[test]
fn golden_nesting_unplaceable() {
    run_case(
        "doc_unplaceable.json",
        "job_unplaceable.json",
        "expected_unplaceable.json",
    );
}

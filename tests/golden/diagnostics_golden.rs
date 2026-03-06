use craftcad_diagnostics::joblog::set_now_fn_for_tests;
use craftcad_diagnostics::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

type ReasonCountsMap = BTreeMap<String, (i64, Option<String>, Option<String>, Option<Severity>)>;

fn repo_root() -> PathBuf {
    let mut p = std::env::current_dir().unwrap();
    for _ in 0..8 {
        if p.join("tests/golden").exists() {
            return p;
        }
        if !p.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn write(path: &Path, s: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, s).unwrap();
}

fn accept() -> bool {
    std::env::var("GOLDEN_ACCEPT").ok().as_deref() == Some("1")
}

fn assert_or_accept(path: &Path, actual: &str) {
    if accept() {
        write(path, actual);
        return;
    }
    let expected = read(path);
    assert_eq!(expected, actual, "golden mismatch: {}", path.display());
}

#[test]
fn diagnostics_golden_outputs_match() {
    let root = repo_root();
    set_now_fn_for_tests(|| "2000-01-01T00:00:02Z".to_string());

    let red = StubRedactor;
    let consent = DefaultDenyConsent;
    let limits = Limits::conservative_default();

    let ctx = JobContext {
        app_version: "1.0.0-test".into(),
        build_id: Some("build-test".into()),
        schema_version: "diycad-1".into(),
        os: "TestOS".into(),
        arch: "x86_64".into(),
        locale: "ja-JP".into(),
        timezone: "Asia/Tokyo".into(),
        determinism_tag: DeterminismTag {
            seed: 42,
            epsilon: 1e-6,
            rounding: "bankers".into(),
            ordering: "btree".into(),
        },
        limits_profile: "default".into(),
    };

    let mut jb = JobLogBuilder::new(ctx, &red, &consent, limits.clone());
    jb.add_input(
        "svg",
        "input-1",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        1234,
        "drag_drop",
    );
    {
        let mut g = jb.begin_step(
            "a1",
            "ImportFile",
            &json!({"format":"svg","input_id":"input-1"}),
        );
        g.add_reason_code("R_IMPORT_WARN");
        g.set_result(StepResultKind::Ok);
    }
    {
        let mut g = jb.begin_step("a2", "RunNest", &json!({"job_id":"job-1"}));
        g.add_reason_code("R_NEST_FAIL");
        g.set_result(StepResultKind::Failed);
    }

    let mut joblog = jb.finish();
    joblog.timeline.steps[0].ts = "2000-01-01T00:00:00Z".to_string();
    joblog.timeline.steps[0].duration_ms = Some(10);
    joblog.timeline.steps[1].ts = "2000-01-01T00:00:01Z".to_string();
    joblog.timeline.steps[1].duration_ms = Some(20);

    joblog.reasons[0].first_ts = "2000-01-01T00:00:02Z".to_string();
    joblog.reasons[0].last_ts = "2000-01-01T00:00:02Z".to_string();
    joblog.reasons[1].first_ts = "2000-01-01T00:00:03Z".to_string();
    joblog.reasons[1].last_ts = "2000-01-01T00:00:03Z".to_string();

    let joblog_json = serde_json::to_string_pretty(&joblog).unwrap() + "\n";
    assert_or_accept(
        &root.join("tests/golden/diagnostics/joblog_sample.json"),
        &joblog_json,
    );

    let mut ob =
        OpLogBuilder::start_session_at("sess-1", "2000-01-01T00:00:00Z", &red, limits.clone());
    ob.record_action(
        "a1",
        "ImportFile",
        &json!({"format":"svg","input_id":"input-1"}),
        &["input-1".into()],
        ActionResult::Ok,
        &["R_IMPORT_WARN".into()],
    );
    ob.record_action(
        "a2",
        "RunNest",
        &json!({"job_id":"job-1"}),
        &["job-1".into()],
        ActionResult::Failed,
        &["R_NEST_FAIL".into()],
    );
    let mut oplog = ob.finish();
    oplog.actions[0].params_hash =
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
    oplog.actions[1].params_hash =
        "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string();

    let oplog_json = serde_json::to_string_pretty(&oplog).unwrap() + "\n";
    assert_or_accept(
        &root.join("tests/golden/diagnostics/oplog_sample.json"),
        &oplog_json,
    );

    let mut counts: ReasonCountsMap = BTreeMap::new();
    for r in &joblog.reasons {
        counts.insert(
            r.code.clone(),
            (
                r.count,
                Some(r.first_ts.clone()),
                Some(r.last_ts.clone()),
                Some(r.severity),
            ),
        );
    }
    let summary = ReasonSummary::from_reason_counts_stable(&counts, &EmptyCatalogLookup, 10);
    let summary_json = serde_json::to_string_pretty(&summary).unwrap() + "\n";
    assert_or_accept(
        &root.join("tests/golden/diagnostics/reason_summary_sample.json"),
        &summary_json,
    );

    let repro = generate_repro_markdown(
        &joblog,
        Some(&oplog),
        Some(ReproArtifacts {
            zip_name: "support-test.zip".to_string(),
            zip_sha256: "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                .to_string(),
        }),
    );
    assert_or_accept(
        &root.join("tests/golden/diagnostics/repro_sample.md"),
        &repro.markdown,
    );

    let manifest = read(&root.join("tests/golden/diagnostics/support_zip_manifest.json"));
    assert!(!manifest.is_empty());
}

use craftcad_diagnostics::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;

struct AllowAllRedactor;
impl Redactor for AllowAllRedactor {
    fn redact_str(&self, s: &str) -> String {
        s.to_string()
    }
    fn redact_json(&self, v: &serde_json::Value) -> serde_json::Value {
        v.clone()
    }
}

#[derive(Default)]
struct ConsentFalse;
impl ConsentProvider for ConsentFalse {
    fn include_project_snapshot(&self) -> bool {
        false
    }
    fn include_inputs_copy(&self) -> bool {
        false
    }
    fn telemetry_opt_in(&self) -> bool {
        false
    }
}

#[test]
fn e2e_support_zip_build_and_validate_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let store = DiagnosticsStore::new(tmp.path()).unwrap();
    let red = AllowAllRedactor;
    let consent = ConsentFalse;
    let limits = Limits::conservative_default();
    let policy = RetentionPolicy::ssot_default();

    let ctx = JobContext {
        app_version: "1.0.0".into(),
        build_id: None,
        schema_version: "diycad-1".into(),
        os: "TestOS".into(),
        arch: "x86_64".into(),
        locale: "ja-JP".into(),
        timezone: "Asia/Tokyo".into(),
        determinism_tag: DeterminismTag {
            seed: 1,
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
        12,
        "drag_drop",
    );
    {
        let mut g = jb.begin_step(
            "a1",
            "ImportFile",
            &json!({"format":"svg","input_id":"input-1"}),
        );
        g.add_reason_code("R1");
        g.set_result(StepResultKind::Ok);
    }
    let joblog = jb.finish();

    let mut ob = OpLogBuilder::start_session("sess", &red, limits.clone());
    ob.record_action(
        "a1",
        "ImportFile",
        &json!({"format":"svg","input_id":"input-1"}),
        &["input-1".into()],
        ActionResult::Ok,
        &["R1".into()],
    );
    let oplog = ob.finish();

    let mut counts: BTreeMap<String, (i64, Option<String>, Option<String>, Option<Severity>)> =
        BTreeMap::new();
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

    let repo_root = std::env::current_dir().unwrap();
    let b = SupportZipBuilder::new()
        .attach_joblog(joblog)
        .attach_oplog(oplog)
        .attach_reason_summary(summary)
        .attach_perf_report(json!({"perf":"dummy"}));

    let res = b.build_into_store(&store, &repo_root, &policy).unwrap();
    assert!(res.path.exists());
    assert_eq!(res.sha256.len(), 64);

    let f = fs::File::open(&res.path).unwrap();
    let mut zr = zip::ZipArchive::new(f).unwrap();
    let mut names = Vec::new();
    for i in 0..zr.len() {
        names.push(zr.by_index(i).unwrap().name().to_string());
    }
    names.sort();

    assert!(names.contains(&"joblog.json".to_string()));
    assert!(names.contains(&"reason_summary.json".to_string()));
    assert!(names.contains(&"ssot_fingerprint.json".to_string()));
    assert!(names.contains(&"perf_report.json".to_string()));
    assert!(names.contains(&"oplog.json".to_string()));
    assert!(!names.contains(&"project_snapshot.diycad".to_string()));
    assert!(!names.iter().any(|n| n.starts_with("inputs/")));
}

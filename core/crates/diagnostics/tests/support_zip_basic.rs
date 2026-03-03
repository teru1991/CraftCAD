use craftcad_diagnostics::*;
use serde_json::json;
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
fn support_zip_contains_required_entries_and_respects_consent_false() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let red = AllowAllRedactor;
    let consent = ConsentFalse;
    let limits = Limits::conservative_default();

    let ctx = JobContext {
        app_version: "1.0.0".into(),
        build_id: None,
        schema_version: "diycad-1".into(),
        os: "TestOS".into(),
        arch: "x86".into(),
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
    {
        let mut g = jb.begin_step("a1", "OpenProject", &json!({"x":1}));
        g.set_result(StepResultKind::Ok);
    }
    let joblog = jb.finish();

    use std::collections::BTreeMap;
    let mut counts = BTreeMap::new();
    counts.insert("A".to_string(), (3, None, None, None));
    let summary = ReasonSummary::from_reason_counts_stable(&counts, &EmptyCatalogLookup, 10);
    let fp = SsotFingerprint::empty();

    let mut b = SupportZipBuilder::new(tmp.path(), limits, &red, &consent).expect("builder");
    b.attach_joblog(joblog)
        .attach_reason_summary(summary)
        .attach_ssot_fingerprint(fp)
        .attach_perf_report(json!({"ok":true}))
        .optionally_attach_project_snapshot(tmp.path().join("should_not_exist.diycad"))
        .optionally_attach_input_copy("a.txt", tmp.path().join("input_a.txt"));

    let res = b.build().expect("build zip");
    assert!(res.size_bytes > 0);
    assert_eq!(res.sha256.len(), 64);
    assert!(res.path.exists());

    let f = fs::File::open(&res.path).expect("open zip");
    let mut zr = zip::ZipArchive::new(f).expect("zip archive");
    let mut names = Vec::new();
    for i in 0..zr.len() {
        names.push(zr.by_index(i).expect("entry").name().to_string());
    }
    names.sort();

    assert!(names.contains(&"joblog.json".to_string()));
    assert!(names.contains(&"reason_summary.json".to_string()));
    assert!(names.contains(&"ssot_fingerprint.json".to_string()));
    assert!(names.contains(&"perf_report.json".to_string()));
    assert!(!names.iter().any(|n| n == "project_snapshot.diycad"));
    assert!(!names.iter().any(|n| n.starts_with("inputs/")));
}

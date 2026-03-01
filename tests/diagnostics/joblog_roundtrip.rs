use craftcad_diagnostics::{JobLog, JobLogContext, JobStep};

#[test]
fn joblog_roundtrip() {
    let mut jl = JobLog::new(JobLogContext {
        app_version: "0.1.0".into(),
        schema_version: "1".into(),
        os: "linux".into(),
        locale: "ja".into(),
        dataset_id: None,
        seed: Some(1),
    });
    jl.push_step(JobStep { timestamp_ms: 1, action_id: "OpenProject".into(), params_hash: "abc".into(), result: "ok".into(), reason_codes: vec![] });
    let s = serde_json::to_string(&jl.finish()).unwrap();
    assert!(s.contains("OpenProject"));
}

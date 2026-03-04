use craftcad_diagnostics::{
    DefaultDenyConsent, DeterminismTag, JobContext, JobLogBuilder, Limits, StepResultKind,
    StubRedactor, SupportZipBuilder,
};

#[test]
fn support_zip_respects_consent_and_is_redacted() {
    let config_dir = tempfile::tempdir().expect("tmp config");
    std::env::set_var("CRAFTCAD_CONFIG_DIR", config_dir.path());

    let ctx = JobContext {
        app_version: "test".into(),
        build_id: None,
        schema_version: "1".into(),
        os: "test".into(),
        arch: "x86_64".into(),
        locale: "ja-JP".into(),
        timezone: "UTC".into(),
        determinism_tag: DeterminismTag {
            seed: 0,
            epsilon: 1e-6,
            rounding: "bankers".into(),
            ordering: "btree".into(),
        },
        limits_profile: "default".into(),
    };
    let redactor = StubRedactor;
    let consent = DefaultDenyConsent;
    let mut jb = JobLogBuilder::new(ctx, &redactor, &consent, Limits::conservative_default());
    {
        let mut step = jb.begin_step(
            "a1",
            "Import",
            &serde_json::json!({
                "email": "user@example.com",
                "note": "my token=SECRET",
                "path": "/Users/alice/projects/private.diycad",
                "x": "Bearer VERYSECRET"
            }),
        );
        step.set_result(StepResultKind::Ok);
    }
    let joblog = jb.finish();

    let out = tempfile::tempdir().expect("tmp out");
    let zip_path = out.path().join("support.zip");
    let zip_path = SupportZipBuilder::new()
        .attach_joblog(joblog)
        .attach_project_snapshot(Some("TOPSECRET".into()))
        .build(&zip_path)
        .expect("zip build");

    let hay = String::from_utf8_lossy(&std::fs::read(&zip_path).expect("read zip"));
    assert!(!hay.contains("user@example.com"));
    assert!(!hay.contains("VERYSECRET"));
    assert!(!hay.contains("/Users/alice"));
    assert!(!hay.contains("project_snapshot.diycad"));
}

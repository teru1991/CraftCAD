use std::io::Read;

use craftcad_diagnostics::{
    DefaultDenyConsent, DeterminismTag, JobContext, JobLogBuilder, Limits, StepResultKind,
    StubRedactor, SupportZipBuilder,
};

#[test]
fn support_zip_contains_no_raw_pii_and_respects_consent_defaults() {
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
                "x": "Bearer SUPERSECRET"
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

    let zip_bytes = std::fs::read(&zip_path).expect("read zip");
    let rdr = std::io::Cursor::new(zip_bytes);
    let mut zip = zip::ZipArchive::new(rdr).expect("open zip bytes");

    let mut names = Vec::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).expect("zip entry");
        let name = f.name().to_string();
        names.push(name);

        let mut buf = Vec::new();
        f.read_to_end(&mut buf).expect("read entry");
        let text = String::from_utf8_lossy(&buf);

        assert!(!text.contains("user@example.com"));
        assert!(!text.contains("Bearer SUPERSECRET"));
        assert!(!text.contains("/Users/alice"));
    }

    assert!(!names.iter().any(|n| n.contains("project_snapshot")));
}

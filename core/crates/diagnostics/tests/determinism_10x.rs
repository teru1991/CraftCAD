use craftcad_diagnostics::joblog::set_now_fn_for_tests;
use craftcad_diagnostics::*;
use serde_json::json;

struct AllowAllRedactor;
impl Redactor for AllowAllRedactor {
    fn redact_str(&self, s: &str) -> String {
        s.to_string()
    }
    fn redact_json(&self, v: &serde_json::Value) -> serde_json::Value {
        v.clone()
    }
}

#[test]
fn diagnostics_outputs_match_over_10_runs() {
    set_now_fn_for_tests(|| "2000-01-01T00:00:00Z".to_string());

    let red = AllowAllRedactor;
    let consent = DefaultDenyConsent;
    let limits = Limits::conservative_default();

    let make = || {
        let ctx = JobContext {
            app_version: "1.0".into(),
            build_id: None,
            schema_version: "s".into(),
            os: "o".into(),
            arch: "a".into(),
            locale: "l".into(),
            timezone: "t".into(),
            determinism_tag: DeterminismTag {
                seed: 7,
                epsilon: 1e-6,
                rounding: "bankers".into(),
                ordering: "btree".into(),
            },
            limits_profile: "default".into(),
        };
        let mut jb = JobLogBuilder::new(ctx, &red, &consent, limits.clone());
        jb.add_input(
            "svg",
            "i1",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            1,
            "drag_drop",
        );
        {
            let mut g = jb.begin_step("a1", "ImportFile", &json!({"x":1}));
            g.add_reason_code("R1");
            g.set_result(StepResultKind::Ok);
        }
        let joblog = jb.finish();

        let mut ob =
            OpLogBuilder::start_session_at("sess", "2000-01-01T00:00:00Z", &red, limits.clone());
        ob.record_action(
            "a1",
            "ImportFile",
            &json!({"x":1}),
            &["i1".into()],
            ActionResult::Ok,
            &["R1".into()],
        );
        let oplog = ob.finish();

        let repro = generate_repro_markdown(&joblog, Some(&oplog), None).markdown;
        (
            serde_json::to_string(&joblog).unwrap(),
            serde_json::to_string(&oplog).unwrap(),
            repro,
        )
    };

    let (j0, o0, r0) = make();
    for _ in 0..10 {
        let (j, o, r) = make();
        assert_eq!(j0, j);
        assert_eq!(o0, o);
        assert_eq!(r0, r);
    }
}

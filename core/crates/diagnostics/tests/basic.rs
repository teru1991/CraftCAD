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
fn joblog_reasons_are_stably_sorted_and_deterministic() {
    let red = AllowAllRedactor;
    let consent = DefaultDenyConsent;
    let limits = Limits::conservative_default();

    let ctx = JobContext {
        app_version: "1.2.3".into(),
        build_id: Some("build-x".into()),
        schema_version: "diycad-7".into(),
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

    let mut b = JobLogBuilder::new(ctx, &red, &consent, limits);
    b.add_reason("Z_CODE", Severity::Warn);
    b.add_reason("A_CODE", Severity::Error);
    b.add_reason("M_CODE", Severity::Info);
    {
        let mut g = b.begin_step("a1", "RunNest", &json!({"foo":"bar"}));
        g.add_reason_code("R2");
        g.add_reason_code("R1");
        g.set_result(StepResultKind::Failed);
    }
    let jl1 = b.finish();

    let ctx2 = jl1.header_to_ctx();
    let mut b2 = JobLogBuilder::new(ctx2, &red, &consent, Limits::conservative_default());
    b2.add_reason("Z_CODE", Severity::Warn);
    b2.add_reason("A_CODE", Severity::Error);
    b2.add_reason("M_CODE", Severity::Info);
    {
        let mut g = b2.begin_step("a1", "RunNest", &json!({"foo":"bar"}));
        g.add_reason_code("R2");
        g.add_reason_code("R1");
        g.set_result(StepResultKind::Failed);
    }
    let jl2 = b2.finish();

    let mut v1 = serde_json::to_value(&jl1).expect("serialize jl1");
    let mut v2 = serde_json::to_value(&jl2).expect("serialize jl2");
    normalize_nondeterministic_time_fields(&mut v1);
    normalize_nondeterministic_time_fields(&mut v2);
    assert_eq!(v1, v2, "joblog JSON shape/order must be deterministic");

    let codes: Vec<_> = jl2.reasons.iter().map(|r| r.code.as_str()).collect();
    assert!(codes.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn oplog_seq_monotonic_no_gaps() {
    let red = AllowAllRedactor;
    let limits = Limits::conservative_default();
    let mut o = OpLogBuilder::start_session("sess-1", &red, limits);
    o.record_action(
        "a1",
        "OpenProject",
        &json!({"k":"v"}),
        &[],
        ActionResult::Ok,
        &[],
    );
    o.record_action(
        "a2",
        "RunNest",
        &json!({"seed":1}),
        &["id2".into(), "id1".into()],
        ActionResult::Failed,
        &["X".into(), "A".into()],
    );
    let op = o.finish();
    assert_eq!(op.actions.len(), 2);
    assert_eq!(op.actions[0].seq, 1);
    assert_eq!(op.actions[1].seq, 2);
    assert_eq!(
        op.actions[1].affected_ids,
        vec!["id1".to_string(), "id2".to_string()]
    );
    assert_eq!(
        op.actions[1].reason_codes,
        vec!["A".to_string(), "X".to_string()]
    );
}

#[test]
fn limits_truncate_inputs_and_params() {
    let red = AllowAllRedactor;
    let consent = DefaultDenyConsent;
    let mut limits = Limits::conservative_default();
    limits.max_inputs = 1;
    limits.max_string_len = 4;

    let ctx = JobContext {
        app_version: "1.0".into(),
        build_id: None,
        schema_version: "s1".into(),
        os: "os".into(),
        arch: "arch".into(),
        locale: "ja-JP".into(),
        timezone: "UTC".into(),
        determinism_tag: DeterminismTag {
            seed: 1,
            epsilon: 0.0001,
            rounding: "r".into(),
            ordering: "o".into(),
        },
        limits_profile: "test".into(),
    };

    let mut b = JobLogBuilder::new(ctx, &red, &consent, limits);
    b.add_input("kind", "id1", "a", 1, "hint");
    b.add_input("kind", "id2", "b", 2, "hint");
    {
        let _g = b.begin_step("a", "k", &json!({"x": "12345678901234567890"}));
    }
    let jl = b.finish();

    assert_eq!(jl.inputs.len(), 1);
    let codes: Vec<&str> = jl.reasons.iter().map(|r| r.code.as_str()).collect();
    assert!(codes.contains(&"DIAG_INPUTS_TRUNCATED"));
    assert!(codes.contains(&"DIAG_PARAMS_TRUNCATED"));
}

fn normalize_nondeterministic_time_fields(v: &mut serde_json::Value) {
    if let Some(steps) = v
        .get_mut("timeline")
        .and_then(|t| t.get_mut("steps"))
        .and_then(|s| s.as_array_mut())
    {
        for step in steps {
            if let Some(obj) = step.as_object_mut() {
                obj.insert(
                    "ts".to_string(),
                    serde_json::Value::String("<ts>".to_string()),
                );
                obj.insert(
                    "duration_ms".to_string(),
                    serde_json::Value::Number(serde_json::Number::from(0)),
                );
            }
        }
    }
    if let Some(reasons) = v.get_mut("reasons").and_then(|r| r.as_array_mut()) {
        for r in reasons {
            if let Some(obj) = r.as_object_mut() {
                obj.insert(
                    "first_ts".to_string(),
                    serde_json::Value::String("<ts>".to_string()),
                );
                obj.insert(
                    "last_ts".to_string(),
                    serde_json::Value::String("<ts>".to_string()),
                );
            }
        }
    }
}

use craftcad_diagnostics::*;
use proptest::prelude::*;
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

proptest! {
    #[test]
    fn joblog_does_not_panic_on_large_params(s in ".*") {
        let red = AllowAllRedactor;
        let consent = DefaultDenyConsent;
        let mut limits = Limits::conservative_default();
        limits.max_string_len = 64;

        let ctx = JobContext {
            app_version: "x".into(),
            build_id: None,
            schema_version: "s".into(),
            os: "o".into(),
            arch: "a".into(),
            locale: "l".into(),
            timezone: "t".into(),
            determinism_tag: DeterminismTag { seed: 1, epsilon: 1e-6, rounding: "bankers".into(), ordering: "btree".into() },
            limits_profile: "default".into(),
        };

        let mut jb = JobLogBuilder::new(ctx, &red, &consent, limits.clone());
        {
            let mut g = jb.begin_step("a1", "Action", &json!({"text": s}));
            g.add_reason_code("R");
            g.set_result(StepResultKind::Ok);
        }
        let jl = jb.finish();
        let js = serde_json::to_string(&jl).unwrap();
        prop_assert!(!js.is_empty());
    }
}

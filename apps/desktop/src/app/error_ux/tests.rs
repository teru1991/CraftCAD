use super::*;
use std::collections::BTreeMap;

fn dummy_catalog() -> reason_catalog::ReasonCatalog {
    use reason_catalog::*;
    let mut reasons = std::collections::BTreeMap::new();
    reasons.insert(
        "E_TEST".to_string(),
        ReasonEntry {
            severity: Some("Error".to_string()),
            title_key: Some("ux.error.test.title".to_string()),
            detail_key: Some("ux.error.test.detail".to_string()),
            why_key: Some("ux.error.test.why".to_string()),
            actions: vec![
                ActionSpec {
                    kind: "OpenDocs".to_string(),
                    args: BTreeMap::from([(
                        "doc_link".to_string(),
                        serde_json::Value::String("ux/test".to_string()),
                    )]),
                    label_key: None,
                },
                ActionSpec {
                    kind: "OpenSettings".to_string(),
                    args: BTreeMap::from([(
                        "section".to_string(),
                        serde_json::Value::String("printing".to_string()),
                    )]),
                    label_key: None,
                },
                ActionSpec {
                    kind: "RetryLastJob".to_string(),
                    args: BTreeMap::new(),
                    label_key: None,
                },
                ActionSpec {
                    kind: "JumpToEntity".to_string(),
                    args: BTreeMap::from([(
                        "entity_id".to_string(),
                        serde_json::Value::String("ent1".to_string()),
                    )]),
                    label_key: None,
                },
            ],
            doc_link: Some("ux/test".to_string()),
        },
    );
    ReasonCatalog {
        version: 1,
        reasons,
    }
}

#[test]
fn mapping_is_deterministic() {
    let cat = dummy_catalog();
    let mapper = error_mapper::ErrorMapper::default();

    let mut ctx = BTreeMap::new();
    ctx.insert("k".to_string(), "v".to_string());
    let a = AppError {
        reason_code: "E_TEST".to_string(),
        severity: Severity::Error,
        context: ctx.clone(),
        job_id: Some("j1".to_string()),
        op_id: Some("o1".to_string()),
    };
    let b = AppError {
        reason_code: "E_TEST".to_string(),
        severity: Severity::Error,
        context: ctx,
        job_id: Some("j1".to_string()),
        op_id: Some("o1".to_string()),
    };

    let ea = mapper.map(&a, &cat);
    let eb = mapper.map(&b, &cat);
    assert_eq!(ea.display_hash, eb.display_hash);
}

#[test]
fn actions_are_limited_to_max3() {
    let cat = dummy_catalog();
    let mapper = error_mapper::ErrorMapper::default();
    let a = AppError {
        reason_code: "E_TEST".to_string(),
        severity: Severity::Error,
        context: BTreeMap::new(),
        job_id: None,
        op_id: None,
    };
    let e = mapper.map(&a, &cat);
    assert!(e.actions.len() <= 3);
}

#[test]
fn fallback_keys_when_unknown_reason() {
    let cat = dummy_catalog();
    let mapper = error_mapper::ErrorMapper::default();
    let a = AppError {
        reason_code: "E_UNKNOWN".to_string(),
        severity: Severity::Error,
        context: BTreeMap::new(),
        job_id: None,
        op_id: None,
    };
    let e = mapper.map(&a, &cat);
    assert_eq!(e.title_key, cat.fallback_title_key());
    assert_eq!(e.detail_key, cat.fallback_detail_key());
}

#[test]
fn action_executor_requires_args() {
    let action = UserFacingAction {
        kind: ActionKind::JumpToEntity,
        label_key: "x".to_string(),
        args: BTreeMap::new(),
    };
    let err = actions::ActionExecutor::exec(&action).unwrap_err();
    let app_err = actions::ActionExecutor::to_app_error(&err, &action);
    assert_eq!(app_err.reason_code, "UI_ACTION_FAILED_MISSING_ARG");
}

#[test]
fn unknown_action_kind_is_ignored() {
    let mut cat = dummy_catalog();
    if let Some(entry) = cat.reasons.get_mut("E_TEST") {
        entry.actions.push(reason_catalog::ActionSpec {
            kind: "UnknownAction".to_string(),
            args: BTreeMap::new(),
            label_key: None,
        });
    }

    let mapper = error_mapper::ErrorMapper::default();
    let a = AppError {
        reason_code: "E_TEST".to_string(),
        severity: Severity::Error,
        context: BTreeMap::new(),
        job_id: None,
        op_id: None,
    };
    let e = mapper.map(&a, &cat);
    assert!(e.actions.iter().all(|x| x.kind.as_str() != "UnknownAction"));
}

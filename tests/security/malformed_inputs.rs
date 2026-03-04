use security::{
    ExternalRefPolicy, Limits, LimitsProfile, PathValidationContext, Sandbox, SvgExternalRefAction,
};

#[test]
fn zip_path_traversal_is_blocked() {
    let sb = Sandbox::new(ExternalRefPolicy::Reject);
    let limits = Limits::load_from_ssot(LimitsProfile::Default).unwrap();
    let ctx = PathValidationContext {
        max_depth: limits.max_path_depth,
    };
    assert!(sb.normalize_rel_path(ctx.clone(), "../x").is_err());
    assert!(sb.normalize_rel_path(ctx.clone(), "/abs/x").is_err());
    assert!(sb.normalize_rel_path(ctx.clone(), "C:\\abs\\x").is_err());
    assert!(sb
        .normalize_rel_path(ctx.clone(), "ok/dir/file.txt")
        .is_ok());
}

#[test]
fn svg_external_refs_are_rejected_or_stripped() {
    let limits = Limits::load_from_ssot(LimitsProfile::Default).unwrap();
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://evil.example/a.png"/></svg>"#;
    let sb_reject = Sandbox::new(ExternalRefPolicy::Reject);
    assert!(sb_reject.handle_svg_external_refs(&limits, svg).is_err());

    let sb_strip = Sandbox::new(ExternalRefPolicy::Strip);
    let (san, action, warn) = sb_strip.handle_svg_external_refs(&limits, svg).unwrap();
    match action {
        SvgExternalRefAction::Stripped(n) => assert!(n >= 1),
        _ => panic!("expected stripped"),
    }
    assert!(warn.is_some());
    assert!(!san.contains("https://evil.example"));
}

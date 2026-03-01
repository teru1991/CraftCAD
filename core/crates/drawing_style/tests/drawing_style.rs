use craftcad_drawing_style::{
    build_render_plan, AnnotationSpec, DimensionSpec, RenderCommand, StylePreset,
};

#[test]
fn default_style_loads() {
    let style = StylePreset::load_default().expect("load style ssot");
    assert_eq!(style.version, "v1");
    assert!(style.line_styles.iter().any(|s| s.name == "solid"));
}

#[test]
fn render_plan_is_deterministic() {
    let style = StylePreset::load_default().expect("style");

    let dims = vec![
        DimensionSpec::Linear {
            id: "d2".to_string(),
            start: [100.0, 10.0],
            end: [200.0, 10.0],
            baseline: false,
        },
        DimensionSpec::Linear {
            id: "d1".to_string(),
            start: [0.0, 10.0],
            end: [50.0, 10.0],
            baseline: true,
        },
    ];
    let ann = vec![AnnotationSpec::Leader {
        id: "a1".to_string(),
        anchor: [5.0, 5.0],
        text_at: [25.0, 15.0],
        value: "穴加工".to_string(),
    }];

    let p1 = build_render_plan(&style, &dims, &ann).expect("plan1");
    let p2 = build_render_plan(&style, &dims, &ann).expect("plan2");
    assert_eq!(p1, p2);

    assert!(matches!(
        p1.commands.first(),
        Some(RenderCommand::DimensionLine { key, .. }) if key == "d1"
    ));
}

#[test]
fn invalid_dimension_returns_reason_code() {
    let style = StylePreset::load_default().expect("style");
    let dims = vec![DimensionSpec::Diameter {
        id: "d_bad".to_string(),
        p1: [0.0, 0.0],
        p2: [0.0, 0.0],
    }];

    let err = build_render_plan(&style, &dims, &[]).expect_err("must fail");
    assert_eq!(err.code, "CAD_DIMENSION_ZERO_LENGTH");
}

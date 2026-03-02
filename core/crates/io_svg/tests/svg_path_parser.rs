use craftcad_io::model::{Entity, Segment2D};
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::ReasonCode;
use craftcad_io::IoEngine;
use craftcad_io_svg::SvgIo;

#[test]
fn svg_path_relative_and_close_are_supported() {
    let svg = r#"<svg><path d="M10 10 l10 0 0 10 z"/></svg>"#;

    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let opts = ImportOptions::default_for_tests();
    let res = eng.import("svg", svg.as_bytes(), &opts).unwrap();

    let paths: Vec<_> = res
        .model
        .entities
        .iter()
        .filter_map(|e| {
            if let Entity::Path(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(paths.len(), 1);
    let segs = &paths[0].segments;
    assert!(segs.len() >= 3);

    match segs[0] {
        Segment2D::Line { a, b } => {
            assert!((a.x - 10.0).abs() < 1e-9 && (a.y - 10.0).abs() < 1e-9);
            assert!((b.x - 20.0).abs() < 1e-9 && (b.y - 10.0).abs() < 1e-9);
        }
        _ => panic!("expected first segment to be Line"),
    }
}

#[test]
fn svg_arc_a_is_converted_to_cubic_and_warns() {
    let svg = r#"<svg><path d="M0 0 A10 10 0 0 1 10 0"/></svg>"#;

    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let mut opts = ImportOptions::default_for_tests();
    opts.enable_approx = false;
    let res = eng.import("svg", svg.as_bytes(), &opts).unwrap();

    let has_cubic = res.model.entities.iter().any(|e| {
        if let Entity::Path(p) = e {
            p.segments
                .iter()
                .any(|s| matches!(s, Segment2D::CubicBezier { .. }))
        } else {
            false
        }
    });
    assert!(
        has_cubic,
        "A command should be converted into CubicBezier segments"
    );

    assert!(
        res.warnings
            .iter()
            .any(|w| w.reason == ReasonCode::IO_CURVE_APPROX_APPLIED),
        "expected IO_CURVE_APPROX_APPLIED warning for A command (best-effort)"
    );
}

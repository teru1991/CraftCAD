use craftcad_io::model::{Entity, Segment2D};
use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_svg::SvgIo;

#[test]
fn svg_transform_translate_applied_to_geometry() {
    let svg =
        r#"<svg><g transform="translate(10,20)"><line x1="0" y1="0" x2="1" y2="0"/></g></svg>"#;

    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let opts = ImportOptions::default_for_tests();
    let res = eng.import("svg", svg.as_bytes(), &opts).unwrap();

    let path = res
        .model
        .entities
        .iter()
        .find_map(|e| {
            if let Entity::Path(p) = e {
                Some(p)
            } else {
                None
            }
        })
        .expect("expected path");

    match path.segments[0] {
        Segment2D::Line { a, b } => {
            assert!((a.x - 10.0).abs() < 1e-9 && (a.y - 20.0).abs() < 1e-9);
            assert!((b.x - 11.0).abs() < 1e-9 && (b.y - 20.0).abs() < 1e-9);
        }
        _ => panic!("expected line segment"),
    }
}

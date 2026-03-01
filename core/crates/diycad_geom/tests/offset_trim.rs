use diycad_geom::{offset, trim_line_to_intersection, EpsilonPolicy, Geom2D, Vec2};

#[test]
fn offset_line_parallel() {
    let g = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 0.0 },
        b: Vec2 { x: 10.0, y: 0.0 },
    };
    let out = offset(&g, 2.0, &EpsilonPolicy::default()).unwrap();
    match out {
        Geom2D::Line { a, b } => {
            assert!((a.y - 2.0).abs() < 1e-9);
            assert!((b.y - 2.0).abs() < 1e-9);
        }
        _ => panic!("expected line"),
    }
}

#[test]
fn trim_line_basic() {
    let target = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 0.0 },
        b: Vec2 { x: 10.0, y: 0.0 },
    };
    let cutter = Geom2D::Line {
        a: Vec2 { x: 5.0, y: -1.0 },
        b: Vec2 { x: 5.0, y: 1.0 },
    };
    let out = trim_line_to_intersection(
        &target,
        &cutter,
        Vec2 { x: 9.0, y: 0.0 },
        &EpsilonPolicy::default(),
        None,
    )
    .unwrap();
    match out {
        Geom2D::Line { a, b } => {
            assert!((a.x - 5.0).abs() < 1e-9);
            assert!((b.x - 10.0).abs() < 1e-9);
        }
        _ => panic!("expected line"),
    }
}

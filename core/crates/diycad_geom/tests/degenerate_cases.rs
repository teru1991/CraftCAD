use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy, Vec2};

#[test]
fn polyline_with_less_than_two_points_is_degenerate() {
    let g = Geom2D::Polyline {
        pts: vec![Vec2 { x: 0.0, y: 0.0 }],
        closed: false,
    };
    let err = project_point(&g, Vec2 { x: 1.0, y: 2.0 }, &EpsilonPolicy::default())
        .expect_err("degenerate");
    assert_eq!(err.code, "GEOM_DEGENERATE");
}

#[test]
fn line_with_same_endpoints_is_degenerate() {
    let g = Geom2D::Line {
        a: Vec2 { x: 1.0, y: 1.0 },
        b: Vec2 { x: 1.0, y: 1.0 },
    };
    let err = split_at(&g, SplitBy::T(0.5), &EpsilonPolicy::default()).expect_err("degenerate");
    assert_eq!(err.code, "GEOM_DEGENERATE");
}

#[test]
fn parallel_lines_no_intersection() {
    let a = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 0.0 },
        b: Vec2 { x: 1.0, y: 0.0 },
    };
    let b = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 1.0 },
        b: Vec2 { x: 1.0, y: 1.0 },
    };
    let err = intersect(&a, &b, &EpsilonPolicy::default()).expect_err("no intersection");
    assert_eq!(err.code, "GEOM_NO_INTERSECTION");
}

#[test]
fn colinear_overlapping_lines_ambiguous() {
    let a = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 0.0 },
        b: Vec2 { x: 2.0, y: 0.0 },
    };
    let b = Geom2D::Line {
        a: Vec2 { x: 1.0, y: 0.0 },
        b: Vec2 { x: 3.0, y: 0.0 },
    };
    let err = intersect(&a, &b, &EpsilonPolicy::default()).expect_err("ambiguous");
    assert_eq!(err.code, "GEOM_INTERSECTION_AMBIGUOUS");
}

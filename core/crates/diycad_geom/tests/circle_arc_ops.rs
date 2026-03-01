use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy, Vec2};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[test]
fn line_circle_tangent_and_secant_classification() {
    let eps = EpsilonPolicy::default();
    let circle = Geom2D::Circle {
        c: Vec2 { x: 0.0, y: 0.0 },
        r: 1.0,
    };
    let tangent = Geom2D::Line {
        a: Vec2 { x: -2.0, y: 1.0 },
        b: Vec2 { x: 2.0, y: 1.0 },
    };
    let secant = Geom2D::Line {
        a: Vec2 { x: -2.0, y: 0.0 },
        b: Vec2 { x: 2.0, y: 0.0 },
    };

    let t_hit = intersect(&tangent, &circle, &eps).expect("tangent");
    assert_eq!(t_hit.points.len(), 1, "debug={}", t_hit.debug);
    assert_eq!(t_hit.debug["classification"], "tangent_or_single");

    let s_hit = intersect(&secant, &circle, &eps).expect("secant");
    assert_eq!(s_hit.points.len(), 2, "debug={}", s_hit.debug);
    assert_eq!(s_hit.debug["classification"], "secant");
}

#[test]
fn line_arc_outside_range_returns_no_intersection() {
    let eps = EpsilonPolicy::default();
    let arc = Geom2D::Arc {
        c: Vec2 { x: 0.0, y: 0.0 },
        r: 2.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::FRAC_PI_2,
        ccw: true,
    };
    let line = Geom2D::Line {
        a: Vec2 { x: -3.0, y: 0.0 },
        b: Vec2 { x: -3.0, y: 3.0 },
    };
    let err = intersect(&line, &arc, &eps).expect_err("outside arc range");
    assert_eq!(err.code, "GEOM_NO_INTERSECTION");
}

#[test]
fn arc_project_and_split_are_stable() {
    let eps = EpsilonPolicy::default();
    let arc = Geom2D::Arc {
        c: Vec2 { x: 1.0, y: -1.0 },
        r: 4.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::PI,
        ccw: true,
    };

    let hit = project_point(&arc, Vec2 { x: 1.0, y: 6.0 }, &eps).expect("project");
    assert!((0.0..=1.0).contains(&hit.t_global));

    let split = split_at(&arc, SplitBy::T(0.5), &eps).expect("split");
    match (split.left, split.right) {
        (
            Geom2D::Arc {
                end_angle: left_end,
                ..
            },
            Geom2D::Arc {
                start_angle: right_start,
                ..
            },
        ) => {
            assert!((left_end - right_start).abs() <= 1e-9);
        }
        _ => panic!("expected arc split"),
    }
}

#[test]
fn seeded_line_arc_configs_are_deterministic() {
    let seed = 20260301_u64;
    let mut rng = StdRng::seed_from_u64(seed);
    let eps = EpsilonPolicy::default();
    let mut signatures_a = Vec::new();
    let mut signatures_b = Vec::new();

    for _ in 0..80 {
        let arc = Geom2D::Arc {
            c: Vec2 {
                x: rng.gen_range(-5.0..5.0),
                y: rng.gen_range(-5.0..5.0),
            },
            r: rng.gen_range(0.05..8.0),
            start_angle: rng.gen_range(-std::f64::consts::PI..std::f64::consts::PI),
            end_angle: rng.gen_range(-std::f64::consts::PI..std::f64::consts::PI),
            ccw: rng.gen_bool(0.5),
        };
        let line = Geom2D::Line {
            a: Vec2 {
                x: rng.gen_range(-12.0..12.0),
                y: rng.gen_range(-12.0..12.0),
            },
            b: Vec2 {
                x: rng.gen_range(-12.0..12.0),
                y: rng.gen_range(-12.0..12.0),
            },
        };

        let s = match intersect(&line, &arc, &eps) {
            Ok(h) => format!("ok:{}:{:?}", h.points.len(), h.points),
            Err(e) => format!("err:{}", e.code),
        };
        signatures_a.push(s);
    }

    let mut rng = StdRng::seed_from_u64(seed);
    for _ in 0..80 {
        let arc = Geom2D::Arc {
            c: Vec2 {
                x: rng.gen_range(-5.0..5.0),
                y: rng.gen_range(-5.0..5.0),
            },
            r: rng.gen_range(0.05..8.0),
            start_angle: rng.gen_range(-std::f64::consts::PI..std::f64::consts::PI),
            end_angle: rng.gen_range(-std::f64::consts::PI..std::f64::consts::PI),
            ccw: rng.gen_bool(0.5),
        };
        let line = Geom2D::Line {
            a: Vec2 {
                x: rng.gen_range(-12.0..12.0),
                y: rng.gen_range(-12.0..12.0),
            },
            b: Vec2 {
                x: rng.gen_range(-12.0..12.0),
                y: rng.gen_range(-12.0..12.0),
            },
        };

        let s = match intersect(&line, &arc, &eps) {
            Ok(h) => format!("ok:{}:{:?}", h.points.len(), h.points),
            Err(e) => format!("err:{}", e.code),
        };
        signatures_b.push(s);
    }

    assert_eq!(signatures_a, signatures_b, "seed={seed} eps={:?}", eps);
}

#[test]
fn degenerate_small_radii_cases_do_not_panic_and_report_reason() {
    let eps = EpsilonPolicy::default();
    let tiny_circle = Geom2D::Circle {
        c: Vec2 { x: 0.0, y: 0.0 },
        r: 0.0,
    };
    let line = Geom2D::Line {
        a: Vec2 { x: -1.0, y: 0.0 },
        b: Vec2 { x: 1.0, y: 0.0 },
    };
    let err = intersect(&line, &tiny_circle, &eps).expect_err("invalid circle radius");
    assert_eq!(err.code, "GEOM_CIRCLE_RADIUS_INVALID");
}

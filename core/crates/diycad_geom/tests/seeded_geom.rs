use diycad_geom::{intersect, project_point, split_at, EpsilonPolicy, Geom2D, SplitBy, Vec2};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[test]
fn seeded_ops_do_not_panic() {
    let seed = 424242_u64;
    let mut rng = StdRng::seed_from_u64(seed);
    let eps = EpsilonPolicy::default();

    for _ in 0..50 {
        let pts: Vec<Vec2> = (0..4)
            .map(|_| Vec2 {
                x: rng.gen_range(-10.0..10.0),
                y: rng.gen_range(-10.0..10.0),
            })
            .collect();
        let poly = Geom2D::Polyline {
            pts: pts.clone(),
            closed: false,
        };
        let line = Geom2D::Line {
            a: Vec2 {
                x: rng.gen_range(-10.0..10.0),
                y: rng.gen_range(-10.0..10.0),
            },
            b: Vec2 {
                x: rng.gen_range(-10.0..10.0),
                y: rng.gen_range(-10.0..10.0),
            },
        };
        let p = Vec2 {
            x: rng.gen_range(-10.0..10.0),
            y: rng.gen_range(-10.0..10.0),
        };

        let _ = intersect(&poly, &line, &eps).map_err(|e| {
            eprintln!(
                "seed={seed} eps={:?} input={} reason={}",
                eps,
                serde_json::json!({"poly": poly, "line": line}),
                e.code
            );
        });
        let _ = project_point(&poly, p, &eps).map_err(|e| {
            eprintln!(
                "seed={seed} eps={:?} input={} reason={}",
                eps,
                serde_json::json!({"poly": poly, "p": p}),
                e.code
            );
        });
        let _ = split_at(&poly, SplitBy::T(0.5), &eps).map_err(|e| {
            eprintln!(
                "seed={seed} eps={:?} input={} reason={}",
                eps,
                serde_json::json!({"poly": poly}),
                e.code
            );
        });
    }
}

#[test]
fn polyline_line_multiple_intersections_are_ambiguous() {
    let poly = Geom2D::Polyline {
        pts: vec![
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 2.0, y: 2.0 },
            Vec2 { x: 4.0, y: 0.0 },
        ],
        closed: false,
    };
    let line = Geom2D::Line {
        a: Vec2 { x: 0.0, y: 1.0 },
        b: Vec2 { x: 4.0, y: 1.0 },
    };
    let hit = intersect(&poly, &line, &EpsilonPolicy::default()).expect("intersections");
    assert!(hit.ambiguous);
    assert!(hit.points.len() >= 2);
}

#[test]
fn polyline_project_returns_global_t_in_range() {
    let poly = Geom2D::Polyline {
        pts: vec![
            Vec2 { x: 0.0, y: 0.0 },
            Vec2 { x: 10.0, y: 0.0 },
            Vec2 { x: 10.0, y: 10.0 },
        ],
        closed: false,
    };
    let hit =
        project_point(&poly, Vec2 { x: 9.0, y: 3.0 }, &EpsilonPolicy::default()).expect("project");
    assert!((0.0..=1.0).contains(&hit.t_global));
}

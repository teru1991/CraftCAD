use diycad_geom::{intersect, EpsilonPolicy, Geom2D, Vec2};
use std::time::Instant;

#[test]
fn intersect_perf_threshold_smoke() {
    let eps = EpsilonPolicy::default();
    let circle = Geom2D::Circle {
        c: Vec2 { x: 0.0, y: 0.0 },
        r: 100.0,
    };

    let start = Instant::now();
    let mut hits = 0usize;
    for i in 0..20_000 {
        let y = -100.0 + (i as f64 % 200.0);
        let line = Geom2D::Line {
            a: Vec2 { x: -200.0, y },
            b: Vec2 { x: 200.0, y },
        };
        if intersect(&line, &circle, &eps).is_ok() {
            hits += 1;
        }
    }
    let elapsed = start.elapsed();
    assert!(hits > 0);
    assert!(
        elapsed.as_millis() < 2500,
        "perf threshold exceeded: {:?}",
        elapsed
    );
}

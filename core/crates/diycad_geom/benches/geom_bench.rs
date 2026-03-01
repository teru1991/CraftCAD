use criterion::{criterion_group, criterion_main, Criterion};
use diycad_geom::{intersect, EpsilonPolicy, Geom2D, Vec2};

fn bench_intersections(c: &mut Criterion) {
    let eps = EpsilonPolicy::default();
    let circle = Geom2D::Circle {
        c: Vec2 { x: 0.0, y: 0.0 },
        r: 100.0,
    };

    c.bench_function("line_circle_intersect_10k", |b| {
        b.iter(|| {
            let mut acc = 0usize;
            for i in 0..10_000 {
                let y = -100.0 + (i as f64 % 200.0);
                let line = Geom2D::Line {
                    a: Vec2 { x: -200.0, y },
                    b: Vec2 { x: 200.0, y },
                };
                if intersect(&line, &circle, &eps).is_ok() {
                    acc += 1;
                }
            }
            acc
        })
    });
}

criterion_group!(benches, bench_intersections);
criterion_main!(benches);

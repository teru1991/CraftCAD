use criterion::{criterion_group, criterion_main, Criterion};

fn bench_io_roundtrip(c: &mut Criterion) {
    c.bench_function("io_roundtrip_sample", |b| {
        b.iter(|| {
            let raw = std::fs::read_to_string("tests/datasets/manifest.json").expect("manifest");
            let _: serde_json::Value = serde_json::from_str(&raw).expect("json");
        })
    });
}

criterion_group!(benches, bench_io_roundtrip);
criterion_main!(benches);

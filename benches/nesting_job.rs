use criterion::{criterion_group, criterion_main, Criterion};

fn bench_nesting_job(c: &mut Criterion) {
    c.bench_function("nesting_manifest_scan", |b| {
        b.iter(|| {
            let bytes = std::fs::read("tests/datasets/manifest.json").expect("manifest");
            assert!(!bytes.is_empty());
        })
    });
}

criterion_group!(benches, bench_nesting_job);
criterion_main!(benches);

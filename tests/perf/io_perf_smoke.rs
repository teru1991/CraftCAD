use std::time::Instant;

use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;
use craftcad_io_json::JsonIo;
use craftcad_io_svg::SvgIo;

fn p95(mut v: Vec<u128>) -> u128 {
    if v.is_empty() {
        return 0;
    }
    v.sort();
    let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
    v[idx.min(v.len() - 1)]
}

#[test]
fn io_perf_budget_smoke() {
    let eng = IoEngine::new()
        .register_importer(Box::new(JsonIo::new()))
        .register_importer(Box::new(SvgIo::new()))
        .register_importer(Box::new(DxfIo::new()));

    let opts = ImportOptions::default_for_tests();
    let json = std::fs::read("tests/golden/io_roundtrip/inputs/json/sample_01.json").unwrap();
    let svg = std::fs::read("tests/golden/io_roundtrip/inputs/svg/sample_01.svg").unwrap();
    let dxf = std::fs::read("tests/golden/io_roundtrip/inputs/dxf/sample_01.dxf").unwrap();

    let mut samples = Vec::new();
    for _ in 0..30 {
        let st = Instant::now();
        let _ = eng.import("json", &json, &opts);
        let _ = eng.import("svg", &svg, &opts);
        let _ = eng.import("dxf", &dxf, &opts);
        samples.push(st.elapsed().as_millis());
    }
    let p = p95(samples);
    assert!(p <= 120, "io perf p95 exceeded budget: {} ms", p);
}

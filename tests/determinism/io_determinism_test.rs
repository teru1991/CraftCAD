use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_json::JsonIo;
use craftcad_io_svg::SvgIo;

fn model_hash(m: &craftcad_io::model::InternalModel) -> String {
    let s = serde_json::to_string(m).unwrap();
    format!("{:x}", md5::compute(s))
}

#[test]
fn determinism_json_svg_import_hash_stable() {
    let eng = IoEngine::new()
        .register_importer(Box::new(JsonIo::new()))
        .register_importer(Box::new(SvgIo::new()));

    let iopts = ImportOptions::default_for_tests();
    let json_bytes = std::fs::read("tests/golden/io_roundtrip/inputs/json/sample_01.json").unwrap();
    let svg_bytes = std::fs::read("tests/golden/io_roundtrip/inputs/svg/sample_01.svg").unwrap();

    let mut hashes = vec![];
    for _ in 0..10 {
        let r = eng.import("json", &json_bytes, &iopts).unwrap();
        hashes.push(model_hash(&r.model));
    }
    assert!(hashes.windows(2).all(|w| w[0] == w[1]));

    let mut hashes2 = vec![];
    for _ in 0..10 {
        let r = eng.import("svg", &svg_bytes, &iopts).unwrap();
        hashes2.push(model_hash(&r.model));
    }
    assert!(hashes2.windows(2).all(|w| w[0] == w[1]));
}

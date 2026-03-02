use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::IoEngine;
use craftcad_io_dxf::DxfIo;
use craftcad_io_svg::SvgIo;

fn model_hash(m: &craftcad_io::model::InternalModel) -> String {
    let s = serde_json::to_string(m).unwrap();
    format!("{:x}", md5::compute(s))
}

#[test]
fn e2e_dxf_import_export_svg_reimport_stable() {
    let eng = IoEngine::new()
        .register_importer(Box::new(DxfIo::new()))
        .register_exporter(Box::new(SvgIo::new()))
        .register_importer(Box::new(SvgIo::new()));

    let iopts = ImportOptions::default_for_tests();
    let eopts = ExportOptions::default_for_tests();

    let dxf = std::fs::read("tests/golden/io_roundtrip/inputs/dxf/sample_01.dxf").unwrap();

    let r1 = eng.import("dxf", &dxf, &iopts).unwrap();
    let h1 = model_hash(&r1.model);

    let out_svg = eng.export("svg", &r1.model, &eopts).unwrap();
    let r2 = eng.import("svg", &out_svg.bytes, &iopts).unwrap();
    let h2 = model_hash(&r2.model);

    assert_eq!(h1, h2, "normalize hash should match after dxf->svg->svg");
}

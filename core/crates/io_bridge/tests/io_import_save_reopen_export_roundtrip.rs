use craftcad_io::options::{ExportOptions, ImportOptions, OriginPolicy};
use craftcad_io::IoEngine;
use craftcad_io_bridge::{
    load_diycad_to_internal_model, save_internal_model_to_diycad, LoadDiycadOptions,
    SaveDiycadOptions,
};
use craftcad_io_dxf::DxfIo;
use craftcad_io_svg::SvgIo;
use std::path::PathBuf;

fn model_hash(m: &craftcad_io::model::InternalModel) -> String {
    let mut lines: Vec<String> = Vec::new();
    for e in &m.entities {
        if let craftcad_io::model::Entity::Path(p) = e {
            for s in &p.segments {
                if let craftcad_io::model::Segment2D::Line { a, b } = s {
                    lines.push(format!("{:.6},{:.6}->{:.6},{:.6}", a.x, a.y, b.x, b.y));
                }
            }
        }
    }
    lines.sort();
    format!("{:x}", md5::compute(lines.join("|")))
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn e2e_dxf_import_save_reopen_export_svg_reimport_hash_match() {
    let eng = IoEngine::new()
        .register_importer(Box::new(DxfIo::new()))
        .register_importer(Box::new(SvgIo::new()))
        .register_exporter(Box::new(SvgIo::new()));

    let mut iopts = ImportOptions::default_for_tests();
    iopts.enable_approx = true;

    let mut eopts = ExportOptions::default_for_tests();
    eopts.enable_approx = true;
    eopts.postprocess = true;
    eopts.origin_policy = OriginPolicy::MoveToZero;

    let dxf =
        std::fs::read(root().join("tests/golden/io_roundtrip/inputs/dxf/sample_01.dxf")).unwrap();
    let r1 = eng.import("dxf", &dxf, &iopts).unwrap();
    let h1 = model_hash(&r1.model);

    let tmp = std::env::temp_dir().join("craftcad_sprint12_e2e.diycad");
    let sopts = SaveDiycadOptions::default_for_tests(tmp.to_str().unwrap());
    let _save_warn = save_internal_model_to_diycad(&r1.model, &sopts).unwrap();

    let lopts = LoadDiycadOptions::default_for_tests(tmp.to_str().unwrap());
    let (m2, _load_warn) = load_diycad_to_internal_model(&lopts).unwrap();

    let out_svg = eng.export("svg", &m2, &eopts).unwrap();
    let r3 = eng.import("svg", &out_svg.bytes, &iopts).unwrap();
    let h3 = model_hash(&r3.model);

    assert_eq!(
        h1, h3,
        "normalize hash should match after dxf->save(.diycad)->reopen->export svg->reimport"
    );
}

use craftcad_io::options::ImportOptions;
use craftcad_io::IoEngine;
use craftcad_io_svg::SvgIo;

#[test]
fn import_bytes_limit_is_enforced_without_crash() {
    let oversized = vec![b'<'; 60 * 1024 * 1024];
    let eng = IoEngine::new().register_importer(Box::new(SvgIo::new()));
    let r = eng.import("svg", &oversized, &ImportOptions::default_for_tests());
    assert!(r.is_err());
    let e = r.err().expect("expected error");
    let s = format!("{e:?}");
    assert!(s.contains("IO_LIMIT") || s.contains("SEC_LIMIT"), "{s}");
}

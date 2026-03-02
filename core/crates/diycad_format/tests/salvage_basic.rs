use diycad_format::{open_package, OpenOptions};
use std::io::Write;
use tempfile::tempdir;
use zip::write::FileOptions;

#[test]
fn opens_when_manifest_missing_by_document_heuristic() {
    let td = tempdir().expect("tmp");
    let p = td.path().join("salvage.diycad");
    let f = std::fs::File::create(&p).expect("create");
    let mut zw = zip::ZipWriter::new(f);
    let zopt = FileOptions::default();

    zw.start_file("foo/document.json", zopt).expect("doc file");
    zw.write_all(
        br#"{"id":"d","name":"n","unit":"mm","entities":[],"parts_index":[],"nest_jobs_index":[]}"#,
    )
    .expect("doc write");
    zw.finish().expect("finish");

    let r = open_package(&p, OpenOptions::default()).expect("open salvage");
    assert_eq!(r.document.id, "d");
    assert!(r
        .warnings
        .iter()
        .any(|w| w.code.as_str() == "OPEN_MANIFEST_MISSING"));
    assert!(r
        .warnings
        .iter()
        .any(|w| w.code.as_str() == "OPEN_DOCUMENT_LOCATE_HEURISTIC_USED"));
}

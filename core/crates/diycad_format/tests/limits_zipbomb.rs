use diycad_format::{open_package, Limits, OpenOptions};
use std::io::Write;
use tempfile::tempdir;
use zip::write::FileOptions;

#[test]
fn rejects_too_many_entries_without_crash() {
    let td = tempdir().expect("tmp");
    let p = td.path().join("bomb.diycad");
    let f = std::fs::File::create(&p).expect("create zip");
    let mut zw = zip::ZipWriter::new(f);
    let zopt = FileOptions::default();

    zw.start_file("manifest.json", zopt).expect("manifest file");
    zw.write_all(br#"{"schema_version":1,"app_version":"t","created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z","unit":"mm","entrypoints":{"document":"document.json"}}"#)
        .expect("manifest write");
    zw.start_file("document.json", zopt).expect("document file");
    zw.write_all(
        br#"{"id":"d","name":"n","unit":"mm","entities":[],"parts_index":[],"nest_jobs_index":[]}"#,
    )
    .expect("doc write");

    for i in 0..2000 {
        zw.start_file(format!("junk/{}.bin", i), zopt)
            .expect("junk file");
        zw.write_all(b"x").expect("junk write");
    }
    zw.finish().expect("finish zip");

    let mut opt = OpenOptions::default();
    opt.limits = Limits {
        max_entries: 100,
        ..Limits::default()
    };

    let err = open_package(&p, opt)
        .expect_err("expected limits error")
        .to_string();
    assert!(
        err.contains("SEC_ZIP_TOO_MANY_ENTRIES") || err.contains("too many entries"),
        "err={}",
        err
    );
}

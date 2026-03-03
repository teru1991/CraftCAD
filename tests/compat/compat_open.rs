use diycad_format::{
    open_package, save_package, Document, Entrypoints, Manifest, NestJob, OpenOptions, SaveOptions,
    Unit,
};
use std::fs;
use std::io::{Read, Write};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;

fn minimal_manifest(schema_version: i64) -> Manifest {
    Manifest {
        schema_version,
        app_version: "0.0.0-test".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        unit: Unit::Mm,
        entrypoints: Entrypoints {
            document: "document.json".to_string(),
        },
        features: None,
        determinism_tag: None,
        content_manifest: None,
    }
}

fn minimal_doc() -> Document {
    Document {
        id: "doc1".to_string(),
        name: "Minimal".to_string(),
        unit: Unit::Mm,
        entities: vec![],
        parts_index: vec![],
        nest_jobs_index: vec![],
        created_at: None,
        updated_at: None,
    }
}

fn rewrite_schema_version(src: &std::path::Path, dst: &std::path::Path, schema_version: i64) {
    let src_file = fs::File::open(src).expect("open src");
    let mut zr = zip::ZipArchive::new(src_file).expect("open zip");
    let dst_file = fs::File::create(dst).expect("create dst");
    let mut zw = zip::ZipWriter::new(dst_file);
    let opt = SimpleFileOptions::default();

    for i in 0..zr.len() {
        let mut f = zr.by_index(i).expect("entry");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("read entry");

        if f.name() == "manifest.json" {
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&bytes).expect("manifest json");
            manifest["schema_version"] = serde_json::json!(schema_version);
            bytes = serde_json::to_vec(&manifest).expect("serialize manifest");
        }

        zw.start_file(f.name(), opt).expect("start file");
        zw.write_all(&bytes).expect("write file");
    }

    zw.finish().expect("finish");
}

#[test]
fn forward_version_opens_readonly_best_effort() {
    let td = tempdir().expect("tmp");
    let ok_path = td.path().join("ok_minimal.diycad");
    save_package(
        &ok_path,
        SaveOptions::default(),
        minimal_manifest(1),
        minimal_doc(),
        vec![],
        Vec::<NestJob>::new(),
        vec![],
    )
    .expect("save");

    let forward_path = td.path().join("forward_v999.diycad");
    rewrite_schema_version(&ok_path, &forward_path, 999);

    let result = open_package(
        &forward_path,
        OpenOptions {
            allow_forward_compat_readonly: true,
            ..OpenOptions::default()
        },
    )
    .expect("open forward");

    assert!(
        result.read_only,
        "forward-incompatible should open read-only"
    );
}

#[test]
fn n_minus_two_placeholder_is_deterministically_skipped_until_supported() {
    let latest_schema_version = 1_i64;
    if latest_schema_version < 3 {
        return;
    }

    panic!("enable real N-2 compat assertion once latest schema >= 3");
}

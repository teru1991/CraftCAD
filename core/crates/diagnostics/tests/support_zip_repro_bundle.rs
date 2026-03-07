use craftcad_diagnostics::{DerivedHashes, SupportZipBuilder};
use craftcad_ssot::{
    deterministic_uuid, FeatureGraphV1, GrainPolicyV1, ManufacturingOutline2dV1,
    MaterialCategoryV1, MaterialV1, PartV1, SsotV1,
};
use std::io::Read;

fn sample_ssot() -> SsotV1 {
    let material_id = deterministic_uuid("material", "supportzip");
    let part_id = deterministic_uuid("part", "supportzip");

    SsotV1::new(
        vec![MaterialV1 {
            material_id,
            category: MaterialCategoryV1::Unspecified,
            name: "mat:/home/alice/private".to_string(),
            thickness_mm: Some(18.0),
            grain_policy: GrainPolicyV1::None,
            kerf_mm: 2.0,
            margin_mm: 5.0,
            estimate_loss_factor: None,
        }],
        vec![PartV1 {
            part_id,
            name: "part:C:\\Users\\alice\\secret".to_string(),
            material_id,
            quantity: 1,
            manufacturing_outline_2d: Some(ManufacturingOutline2dV1 {
                min_x: 0.0,
                min_y: 0.0,
                max_x: 100.0,
                max_y: 50.0,
            }),
            thickness_mm: Some(18.0),
            grain_direction: None,
            labels: vec![],
            feature_ids: vec![],
        }],
        FeatureGraphV1::empty(),
    )
    .canonicalize()
}

#[test]
fn support_zip_contains_repro_bundle_files_and_redacts_paths() {
    let config_dir = tempfile::tempdir().expect("config dir");
    std::env::set_var("CRAFTCAD_CONFIG_DIR", config_dir.path());

    let out = tempfile::tempdir().expect("tmp out");
    let zip_path = out.path().join("support.zip");

    let ssot = sample_ssot();
    let builder = SupportZipBuilder::new()
        .support_zip_add_repro_bundle(&ssot, None::<DerivedHashes>, None, Some("test-app"))
        .expect("repro bundle attached");
    let zip_path = builder.build(&zip_path).expect("build zip");

    let bytes = std::fs::read(zip_path).expect("zip bytes");
    let cursor = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor).expect("zip open");

    let mut names = Vec::new();
    for i in 0..zip.len() {
        names.push(zip.by_index(i).expect("entry").name().to_string());
    }

    assert!(names.contains(&"repro/ssot_snapshot.json".to_string()));
    assert!(names.contains(&"repro/derived_hashes.json".to_string()));
    assert!(names.contains(&"repro/env.json".to_string()));
    let mut names_sorted = names.clone();
    names_sorted.sort();
    println!("zip_entries={:?}", names_sorted);

    let mut ssot_text = String::new();
    zip.by_name("repro/ssot_snapshot.json")
        .expect("ssot snapshot")
        .read_to_string(&mut ssot_text)
        .expect("read ssot snapshot");
    let mut hashes_text = String::new();
    zip.by_name("repro/derived_hashes.json")
        .expect("derived hashes")
        .read_to_string(&mut hashes_text)
        .expect("read hashes");
    let mut env_text = String::new();
    zip.by_name("repro/env.json")
        .expect("env")
        .read_to_string(&mut env_text)
        .expect("read env");

    let hashes_json: serde_json::Value = serde_json::from_str(&hashes_text).expect("hashes json");
    for key in [
        "projection_front",
        "projection_top",
        "projection_side",
        "estimate",
        "fastener_bom",
        "mfg_hints",
        "viewpack",
    ] {
        assert!(hashes_json
            .get(key)
            .and_then(serde_json::Value::as_str)
            .is_some());
    }

    let env_json: serde_json::Value = serde_json::from_str(&env_text).expect("env json");
    for key in ["git_sha", "rustc_version", "os", "app_version"] {
        assert!(env_json.get(key).is_some());
    }

    assert!(!ssot_text.contains("/home/"));
    assert!(!ssot_text.contains("C:\\\\"));
}

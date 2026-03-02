use craftcad_library::index::{rebuild_index, search, AssetKind, AssetMeta};

#[test]
fn search_scores_id_and_tags() {
    let assets = vec![
        AssetMeta {
            kind: AssetKind::PresetMaterial,
            id: "plywood_18mm".into(),
            version: "1.0.0".into(),
            display_name_key: Some("preset.material.plywood_18mm".into()),
            tags: vec!["wood".into(), "plywood".into(), "18mm".into()],
            source: "builtin".into(),
        },
        AssetMeta {
            kind: AssetKind::PresetProcess,
            id: "laser_basic".into(),
            version: "1.0.0".into(),
            display_name_key: Some("preset.process.laser_basic".into()),
            tags: vec!["process".into(), "laser".into()],
            source: "builtin".into(),
        },
    ];
    let idx = rebuild_index(0, assets).unwrap();

    let h1 = search(&idx, "plywood_18mm", 10);
    assert_eq!(h1.len(), 1);
    assert!(h1[0].score > 1_000_000);

    let h2 = search(&idx, "laser", 10);
    assert_eq!(h2.len(), 1);
}

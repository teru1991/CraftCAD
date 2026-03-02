use craftcad_library::index::{rebuild_index, AssetKind, AssetMeta};
use craftcad_library::store::{load_index_or_rebuild, LibraryLayout};

#[test]
fn load_index_rebuild_when_missing() {
    let dir = tempfile::tempdir().unwrap();
    let layout = LibraryLayout {
        root: dir.path().to_path_buf(),
    };

    let (idx, warnings) = load_index_or_rebuild(&layout, || {
        let assets = vec![AssetMeta {
            kind: AssetKind::Template,
            id: "x".into(),
            version: "1.0.0".into(),
            display_name_key: None,
            tags: vec!["t".into()],
            source: "builtin".into(),
        }];
        rebuild_index(0, assets)
    })
    .unwrap();

    assert_eq!(idx.schema_version, 1);
    assert!(!warnings.is_empty());
}

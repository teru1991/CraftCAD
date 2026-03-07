use craftcad_artifact_store::{ArtifactEntryV1, ArtifactStoreV1};
use craftcad_dirty_deps::ArtifactKind;

#[test]
fn canonicalize_sorts_and_last_wins() {
    let store = ArtifactStoreV1 {
        schema_version: 1,
        entries: vec![
            ArtifactEntryV1 {
                kind: ArtifactKind::ViewpackV1,
                schema_version: 1,
                sha256_hex: "old".to_string(),
                bytes: vec![1],
            },
            ArtifactEntryV1 {
                kind: ArtifactKind::EstimateLiteV1,
                schema_version: 1,
                sha256_hex: "est".to_string(),
                bytes: vec![2],
            },
            ArtifactEntryV1 {
                kind: ArtifactKind::ViewpackV1,
                schema_version: 1,
                sha256_hex: "new".to_string(),
                bytes: vec![3],
            },
        ],
    }
    .canonicalize();

    assert_eq!(store.entries.len(), 2);
    assert_eq!(store.entries[0].kind, ArtifactKind::EstimateLiteV1);
    assert_eq!(store.entries[1].kind, ArtifactKind::ViewpackV1);
    assert_eq!(store.entries[1].sha256_hex, "new");
}

#[test]
fn upsert_keeps_canonical_order() {
    let mut store = ArtifactStoreV1::default();
    store.upsert(ArtifactEntryV1 {
        kind: ArtifactKind::ViewpackV1,
        schema_version: 1,
        sha256_hex: "v".to_string(),
        bytes: vec![1],
    });
    store.upsert(ArtifactEntryV1 {
        kind: ArtifactKind::EstimateLiteV1,
        schema_version: 1,
        sha256_hex: "e".to_string(),
        bytes: vec![2],
    });

    assert_eq!(store.entries.len(), 2);
    assert_eq!(store.entries[0].kind, ArtifactKind::EstimateLiteV1);
}

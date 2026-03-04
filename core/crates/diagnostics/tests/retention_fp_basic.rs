use craftcad_diagnostics::*;
use std::fs;

#[test]
fn ssot_fingerprint_is_deterministic_and_sorted() {
    let root = std::env::current_dir().unwrap();
    let fp1 = SsotFingerprint::compute(&root);
    let fp2 = SsotFingerprint::compute(&root);
    let j1 = serde_json::to_string(&fp1).unwrap();
    let j2 = serde_json::to_string(&fp2).unwrap();
    assert_eq!(j1, j2);
    assert!(fp1.items.windows(2).all(|w| w[0].path <= w[1].path));
}

#[test]
fn retention_policy_validate_ranges() {
    let p = RetentionPolicy::ssot_default();
    p.validate().unwrap();
}

#[test]
fn diagnostics_store_cleanup_deletes_oldest_deterministically() {
    let tmp = tempfile::tempdir().unwrap();
    let store = DiagnosticsStore::new(tmp.path()).unwrap();

    for _ in 0..3 {
        let (id, dir) = store.create_item_dir().unwrap();
        fs::write(dir.join("dummy.txt"), b"123").unwrap();
        let entry = StoreIndexEntry {
            id: id.clone(),
            created_at: "2000-01-01T00:00:00Z".to_string(),
            rel_dir: format!("items/{id}"),
            zip_rel_path: None,
            size_bytes: Some(3),
        };
        store.append_index(&entry).unwrap();
    }

    let pol = RetentionPolicy {
        default_keep_days: 1,
        max_total_bytes: 64 * 1024 * 1024,
        max_items: 100,
    };
    let res = store.cleanup(&pol).unwrap();
    assert_eq!(res.deleted_ids.len(), 3);
    let mut sorted = res.deleted_ids.clone();
    sorted.sort();
    assert_eq!(res.deleted_ids, sorted);
}

#![forbid(unsafe_code)]

use craftcad_cache::{CacheEntry, CacheKey, CacheKeyMaterial, CacheStore, DefaultCachePolicy, Sha256Hex};
use serde_json::json;

fn keymat(ssot_hash: &str, inputs: &str, opts: &str) -> CacheKeyMaterial {
    CacheKeyMaterial {
        dataset_id: "heavy_sample_v1".to_string(),
        seed: 0,
        eps: "1e-6".to_string(),
        round_step: "1e-4".to_string(),
        inputs_sha256: inputs.to_string(),
        options_sha256: opts.to_string(),
        ssot_hash: ssot_hash.to_string(),
        schema_version: 1,
    }
}

#[test]
fn cache_key_is_deterministic_and_changes_on_material_change() {
    let m1 = keymat("aaaaaaaa", "bbbbbbbb", "cccccccc");
    let k1 = CacheKey::new(&m1).unwrap();
    let k1b = CacheKey::new(&m1).unwrap();
    assert_eq!(k1.as_str(), k1b.as_str());

    let mut m2 = m1.clone();
    m2.ssot_hash = "dddddddd".to_string();
    let k2 = CacheKey::new(&m2).unwrap();
    assert_ne!(k1.as_str(), k2.as_str());

    let mut m3 = m1.clone();
    m3.inputs_sha256 = "eeeeeeee".to_string();
    let k3 = CacheKey::new(&m3).unwrap();
    assert_ne!(k1.as_str(), k3.as_str());

    let mut m4 = m1.clone();
    m4.options_sha256 = "ffffffff".to_string();
    let k4 = CacheKey::new(&m4).unwrap();
    assert_ne!(k1.as_str(), k4.as_str());

    let mut m5 = m1;
    m5.schema_version = 2;
    let k5 = CacheKey::new(&m5).unwrap();
    assert_ne!(k1.as_str(), k5.as_str());
}

#[test]
fn canonical_json_hash_is_order_independent_for_objects() {
    let v1 = json!({"b": 2, "a": {"y": 2, "x": 1}});
    let v2 = json!({"a": {"x": 1, "y": 2}, "b": 2});
    assert_eq!(Sha256Hex::from_json_value(v1), Sha256Hex::from_json_value(v2));
}

#[test]
fn lru_eviction_respects_total_bytes_and_is_silent_by_default() {
    let mut p = DefaultCachePolicy::conservative();
    p.max_total_bytes = 3;
    p.max_entry_bytes = 3;
    p.max_entries = 10;

    let s = CacheStore::new(p);

    let k1 = CacheKey::new(&keymat("aaaaaaaa", "aa111111", "ab111111")).unwrap();
    let k2 = CacheKey::new(&keymat("aaaaaaaa", "aa222222", "ab111111")).unwrap();
    let k3 = CacheKey::new(&keymat("aaaaaaaa", "aa333333", "ab111111")).unwrap();

    s.put(&k1, CacheEntry { bytes: 1, value: json!({"v": 1}) }).unwrap();
    s.put(&k2, CacheEntry { bytes: 1, value: json!({"v": 2}) }).unwrap();

    assert!(s.get(&k1).unwrap().is_some());

    s.put(&k3, CacheEntry { bytes: 1, value: json!({"v": 3}) }).unwrap();

    let k4 = CacheKey::new(&keymat("aaaaaaaa", "aa444444", "ab111111")).unwrap();
    s.put(&k4, CacheEntry { bytes: 1, value: json!({"v": 4}) }).unwrap();

    let v1 = s.get(&k1).unwrap();
    let v2 = s.get(&k2).unwrap();
    let v3 = s.get(&k3).unwrap();
    let v4 = s.get(&k4).unwrap();

    assert!(v1.is_some(), "k1 should remain (recently used)");
    assert!(v3.is_some(), "k3 should remain");
    assert!(v4.is_some(), "k4 should remain");
    assert!(v2.is_none(), "k2 should be evicted as LRU");

    let ws = s.drain_warnings().unwrap();
    assert!(!ws.is_empty(), "eviction should be recorded for diagnostics");
}

#[test]
fn invalidation_is_natural_via_key_material() {
    let p = DefaultCachePolicy::conservative();
    let s = CacheStore::new(p);

    let k_a = CacheKey::new(&keymat("aaaaaaaa", "aaaaaaaa", "abaaaaaa")).unwrap();
    let k_b = CacheKey::new(&keymat("bbbbbbbb", "aaaaaaaa", "abaaaaaa")).unwrap();

    s.put(&k_a, CacheEntry { bytes: 10, value: json!({"ok": true}) }).unwrap();

    assert!(s.get(&k_a).unwrap().is_some());
    assert!(s.get(&k_b).unwrap().is_none(), "ssot hash changed => different key => naturally invalidated");
}

#[test]
fn oversized_entry_is_rejected_without_insertion() {
    let mut p = DefaultCachePolicy::conservative();
    p.max_entry_bytes = 1;
    let s = CacheStore::new(p);

    let k = CacheKey::new(&keymat("aaaaaaaa", "abcdef12", "abcdef34")).unwrap();
    s.put(&k, CacheEntry { bytes: 2, value: json!({"too": "big"}) }).unwrap();

    assert!(s.get(&k).unwrap().is_none());
}

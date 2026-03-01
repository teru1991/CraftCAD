use craftcad_i18n::locale_dict;
use std::collections::BTreeSet;

#[test]
fn reason_codes_unique_and_have_i18n_keys() {
    let raw = std::fs::read_to_string("../errors/reason_codes.json").expect("read reason codes");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let arr = v["codes"].as_array().expect("codes array");

    let mut set = BTreeSet::new();
    let ja = locale_dict("ja-JP").expect("ja dict");
    for c in arr {
        let code = c.as_str().expect("code string");
        assert!(code.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_' || ch.is_ascii_digit()));
        assert!(set.insert(code.to_string()), "duplicate code: {code}");
        let key = code.to_lowercase();
        assert!(ja.contains_key(&key), "missing ja-JP i18n key: {key}");
    }
}

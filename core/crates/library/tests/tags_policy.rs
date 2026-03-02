use craftcad_library::tags::{load_tags_policy_from_repo_root, normalize_and_validate_tags};

#[test]
fn tags_normalize_and_validate() {
    let policy = load_tags_policy_from_repo_root(None).unwrap();
    let tags = vec![
        " Wood ".to_string(),
        "wood".to_string(),
        "LEATHER".to_string(),
        "a\u{3000}b".to_string(),
    ];
    let (out, warnings) = normalize_and_validate_tags(&tags, &policy).unwrap();
    assert!(out.contains(&"wood".to_string()));
    assert!(out.contains(&"leather".to_string()));
    assert!(out.contains(&"a b".to_string()));
    assert!(!warnings.is_empty());
}

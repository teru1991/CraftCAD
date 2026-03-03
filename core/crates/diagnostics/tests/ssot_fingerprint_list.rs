use craftcad_diagnostics::ssot_fingerprint::ssot_fingerprint_paths;

#[test]
fn ssot_fingerprint_path_list_is_stable() {
    let paths = ssot_fingerprint_paths();
    assert!(paths.len() >= 7, "diagnostics ssot must be included");
    let s: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    assert!(s
        .iter()
        .any(|p| p.ends_with("docs/specs/diagnostics/joblog.schema.json")));
    assert!(s
        .iter()
        .any(|p| p.ends_with("docs/specs/diagnostics/oplog.schema.json")));
}

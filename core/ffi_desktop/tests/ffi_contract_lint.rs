use sha2::{Digest, Sha256};

#[test]
fn exported_symbols_snapshot_matches_docs_hash() {
    let joined = craftcad_ffi_desktop::EXPORTED_SYMBOLS.join("\n") + "\n";
    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    let actual = format!("{:x}", hasher.finalize());

    let doc = std::fs::read_to_string("../docs/specs/system/ffi_contract.md")
        .or_else(|_| std::fs::read_to_string("../../docs/specs/system/ffi_contract.md"))
        .expect("read ffi contract doc");
    let line = doc
        .lines()
        .find(|l| l.contains("ffi_symbols_sha256"))
        .expect("hash line present");
    let expected = line
        .split('`')
        .nth(1)
        .expect("hash in backticks");

    assert_eq!(actual, expected, "ffi symbol snapshot drift");
}

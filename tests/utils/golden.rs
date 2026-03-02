use std::fs;
use std::path::Path;

pub fn read_bytes(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

pub fn read_string(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

pub fn write_string(path: &str, s: &str) {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("failed to create dirs: {}", e));
    }
    fs::write(path, s).unwrap_or_else(|e| panic!("failed to write {}: {}", path, e));
}

pub fn accept_enabled() -> bool {
    std::env::var("GOLDEN_ACCEPT").ok().as_deref() == Some("1")
}

pub fn assert_or_accept(path: &str, got: &str) {
    if accept_enabled() {
        write_string(path, got);
        return;
    }
    let exp = read_string(path);
    assert_eq!(exp, got, "golden mismatch: {}", path);
}

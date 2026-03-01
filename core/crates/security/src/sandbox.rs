use std::path::{Path, PathBuf};

pub fn ensure_within(base: &Path, candidate: &Path) -> Result<PathBuf, String> {
    let full = candidate
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize: {e}"))?;
    let base = base
        .canonicalize()
        .map_err(|e| format!("failed to canonicalize base: {e}"))?;
    if full.starts_with(&base) {
        Ok(full)
    } else {
        Err("path escapes sandbox".to_string())
    }
}

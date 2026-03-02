use crate::ReasonCode;

#[derive(Debug, Clone)]
pub struct Limits {
    pub max_entries: usize,
    pub max_total_uncompressed: u64,
    pub max_entry_uncompressed: u64,
    pub max_path_len: usize,
    pub max_path_depth: usize,
    pub max_parts: usize,
    pub max_nest_jobs: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_entries: 50_000,
            max_total_uncompressed: 2 * 1024 * 1024 * 1024, // 2GiB
            max_entry_uncompressed: 256 * 1024 * 1024,      // 256MiB
            max_path_len: 240,
            max_path_depth: 16,
            max_parts: 200_000,
            max_nest_jobs: 50_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LimitViolation {
    pub code: ReasonCode,
    pub message: String,
}

fn is_abs_like(p: &str) -> bool {
    p.starts_with('/') || p.contains(':')
}

fn depth_of(p: &str) -> usize {
    p.split('/').filter(|s| !s.is_empty()).count()
}

pub fn validate_entry_path(lim: &Limits, raw: &str) -> Result<String, LimitViolation> {
    if raw.is_empty() {
        return Err(LimitViolation {
            code: ReasonCode::SecZipInvalidEntryName,
            message: "empty entry name".to_string(),
        });
    }
    if raw.contains('\\') {
        return Err(LimitViolation {
            code: ReasonCode::SecZipInvalidEntryName,
            message: "backslash is not allowed".to_string(),
        });
    }
    if is_abs_like(raw) {
        return Err(LimitViolation {
            code: ReasonCode::SecZipAbsolutePath,
            message: format!("absolute-like path rejected: {}", raw),
        });
    }
    if raw.len() > lim.max_path_len {
        return Err(LimitViolation {
            code: ReasonCode::SecZipPathTooLong,
            message: format!("path too long: {} > {}", raw.len(), lim.max_path_len),
        });
    }
    let depth = depth_of(raw);
    if depth > lim.max_path_depth {
        return Err(LimitViolation {
            code: ReasonCode::SecZipPathTooDeep,
            message: format!("path too deep: {} > {}", depth, lim.max_path_depth),
        });
    }
    for seg in raw.split('/') {
        if seg == ".." {
            return Err(LimitViolation {
                code: ReasonCode::SecZipTraversal,
                message: format!("traversal segment rejected: {}", raw),
            });
        }
    }
    Ok(raw.trim_start_matches("./").to_string())
}

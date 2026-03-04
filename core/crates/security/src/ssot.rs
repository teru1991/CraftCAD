use std::path::{Path, PathBuf};

use crate::reasons::{SecCode, SecError, SecResult};

#[derive(Debug, Clone)]
pub struct RepoRoot(PathBuf);

impl RepoRoot {
    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn discover() -> SecResult<Self> {
        if let Ok(v) = std::env::var("CRAFTCAD_REPO_ROOT") {
            let p = PathBuf::from(v);
            if p.join("docs/specs/security").is_dir() {
                return Ok(Self(p));
            }
        }
        let mut cur = std::env::current_dir().map_err(|e| {
            SecError::new(SecCode::SecSsotNotFound, format!("current_dir failed: {e}"))
        })?;
        for _ in 0..64 {
            if cur.join("docs/specs/security").is_dir() {
                return Ok(Self(cur));
            }
            if !cur.pop() {
                break;
            }
        }
        Err(SecError::new(
            SecCode::SecSsotNotFound,
            "docs/specs/security not found (set CRAFTCAD_REPO_ROOT)",
        ))
    }
}

#[derive(Debug, Clone)]
pub struct SsotPaths {
    pub dir_security: PathBuf,
    pub limits_json: PathBuf,
    pub redaction_rules_json: PathBuf,
    pub consent_schema_json: PathBuf,
    pub limits_schema_json: PathBuf,
    pub redaction_schema_json: PathBuf,
}

impl SsotPaths {
    pub fn from_repo_root(root: &RepoRoot) -> Self {
        let dir = root.path().join("docs/specs/security");
        Self {
            dir_security: dir.clone(),
            limits_json: dir.join("limits.json"),
            redaction_rules_json: dir.join("redaction_rules.json"),
            consent_schema_json: dir.join("consent.schema.json"),
            limits_schema_json: dir.join("limits.schema.json"),
            redaction_schema_json: dir.join("redaction.schema.json"),
        }
    }
}

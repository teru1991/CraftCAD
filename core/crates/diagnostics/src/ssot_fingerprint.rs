use crate::reasons::diag_codes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SsotFingerprint {
    pub items: Vec<SsotFingerprintItem>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SsotFingerprintItem {
    pub path: String,
    pub sha256: String,
}

impl SsotFingerprint {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn compute(repo_root: impl AsRef<Path>) -> Self {
        let root = repo_root.as_ref();
        let mut warnings: Vec<String> = Vec::new();

        let mut items = Vec::new();
        for rel in ssot_fingerprint_paths() {
            let abs = root.join(&rel);
            match sha256_file(&abs) {
                Ok(sha) => items.push(SsotFingerprintItem {
                    path: rel.to_string_lossy().to_string(),
                    sha256: sha,
                }),
                Err(_) => {
                    if !warnings
                        .iter()
                        .any(|w| w == diag_codes::DIAG_SSOT_FINGERPRINT_PARTIAL)
                    {
                        warnings.push(diag_codes::DIAG_SSOT_FINGERPRINT_PARTIAL.to_string());
                    }
                    items.push(SsotFingerprintItem {
                        path: rel.to_string_lossy().to_string(),
                        sha256: String::new(),
                    });
                }
            }
        }

        items.sort_by(|a, b| a.path.cmp(&b.path));
        warnings.sort();

        Self { items, warnings }
    }
}

/// Fixed list of SSOT files included in fingerprint.
/// IMPORTANT: modify only with explicit spec change + review (Step4 verification evidence required).
pub fn ssot_fingerprint_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("docs/specs/diagnostics/joblog.schema.json"),
        PathBuf::from("docs/specs/diagnostics/oplog.schema.json"),
        PathBuf::from("docs/specs/diagnostics/support_zip.md"),
        PathBuf::from("docs/specs/diagnostics/retention_policy.md"),
        PathBuf::from("docs/specs/diagnostics/privacy.md"),
        PathBuf::from("docs/specs/diagnostics/repro_template.md"),
        PathBuf::from("docs/specs/diagnostics/README.md"),
        PathBuf::from("docs/specs/security/limits.md"),
        PathBuf::from("docs/specs/security/redaction.md"),
        PathBuf::from("docs/specs/determinism/README.md"),
        PathBuf::from("docs/specs/reasons/reason_catalog.json"),
        PathBuf::from("docs/specs/io/support_matrix.json"),
        PathBuf::from("docs/specs/presets/README.md"),
        PathBuf::from("docs/specs/templates/README.md"),
    ]
}

fn sha256_file(p: &Path) -> io::Result<String> {
    let data = fs::read(p)?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(hex::encode(h.finalize()))
}

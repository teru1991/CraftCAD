use crate::{
    AppWarning, ContentEntry, ContentManifest, Manifest, ReasonCode, SalvageActionHint, WarningKind,
};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub fn build_content_manifest(entries: &[(String, Vec<u8>)]) -> ContentManifest {
    let mut out: Vec<ContentEntry> = Vec::with_capacity(entries.len());
    for (path, bytes) in entries {
        out.push(ContentEntry {
            path: path.clone(),
            size: bytes.len() as u64,
            sha256: sha256_hex(bytes),
        });
    }
    ContentManifest { entries: out }
}

pub fn verify_content_manifest(
    manifest: &Manifest,
    fetch_bytes: &mut dyn FnMut(&str) -> Result<Option<Vec<u8>>>,
) -> Result<(bool, Vec<AppWarning>, Vec<SalvageActionHint>)> {
    let mut warnings: Vec<AppWarning> = Vec::new();
    let mut salvage: Vec<SalvageActionHint> = Vec::new();
    let mut read_only = false;

    let cm = match &manifest.content_manifest {
        Some(c) => c,
        None => {
            warnings.push(AppWarning {
                code: ReasonCode::SaveIntegrityManifestMissing,
                path: Some("manifest.json".to_string()),
                kind: WarningKind::Warning,
                message: "content_manifest missing; integrity verification skipped (compat-first)"
                    .to_string(),
            });
            return Ok((false, warnings, salvage));
        }
    };

    for e in &cm.entries {
        let bytes_opt =
            fetch_bytes(&e.path).with_context(|| format!("fetch_bytes failed: {}", e.path))?;
        let bytes = match bytes_opt {
            Some(b) => b,
            None => {
                read_only = true;
                warnings.push(AppWarning {
                    code: ReasonCode::SaveIntegrityEntryMissing,
                    path: Some(e.path.clone()),
                    kind: WarningKind::Error,
                    message: "entry listed in content_manifest is missing".to_string(),
                });
                continue;
            }
        };
        if bytes.len() as u64 != e.size {
            read_only = true;
            warnings.push(AppWarning {
                code: ReasonCode::SaveIntegritySizeMismatch,
                path: Some(e.path.clone()),
                kind: WarningKind::Error,
                message: format!("size mismatch: actual={} expected={}", bytes.len(), e.size),
            });
            continue;
        }
        if sha256_hex(&bytes) != e.sha256 {
            read_only = true;
            warnings.push(AppWarning {
                code: ReasonCode::SaveIntegrityShaMismatch,
                path: Some(e.path.clone()),
                kind: WarningKind::Error,
                message: "sha256 mismatch".to_string(),
            });
        }
    }

    if read_only {
        salvage.push(SalvageActionHint::ExportSalvagedDocument);
        salvage.push(SalvageActionHint::ExportSalvagedParts);
        salvage.push(SalvageActionHint::GenerateDiagnosticsZip);
        salvage.push(SalvageActionHint::ResaveAsNewProject);
    }

    Ok((read_only, warnings, salvage))
}

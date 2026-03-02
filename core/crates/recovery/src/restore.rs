use crate::reasons::RecoveryReason;
use anyhow::{anyhow, Context, Result};
use diycad_format::{open_package, OpenOptions, OpenResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GenerationInfo {
    pub path: PathBuf,
    pub size: u64,
}

pub fn list_generations(dir: &Path) -> Result<Vec<GenerationInfo>> {
    let mut out: Vec<GenerationInfo> = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }

    for e in fs::read_dir(dir).with_context(|| {
        format!(
            "{}: {}",
            RecoveryReason::RestoreListReadFailed.as_str(),
            dir.display()
        )
    })? {
        let p = e?.path();
        if p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("diycad") {
            let size = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            out.push(GenerationInfo { path: p, size });
        }
    }

    out.sort_by(|a, b| b.path.file_name().cmp(&a.path.file_name()));
    Ok(out)
}

pub fn restore_latest_best_effort(dir: &Path, mut opt: OpenOptions) -> Result<OpenResult> {
    let gens = list_generations(dir)?;
    if gens.is_empty() {
        return Err(anyhow!("{}", RecoveryReason::RestoreNoGenerations.as_str()));
    }

    opt.allow_salvage = true;
    for g in gens {
        if let Ok(r) = open_package(&g.path, opt.clone()) {
            return Ok(r);
        }
    }

    Err(anyhow!("{}", RecoveryReason::RestoreOpenFailed.as_str()))
}

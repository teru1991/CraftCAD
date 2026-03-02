use crate::{policy::RecoveryPolicy, reasons::RecoveryReason};
use anyhow::{anyhow, Context, Result};
use diycad_format::{save_package, Document, Manifest, NestJob, Part, SaveOptions};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;

fn now_compact() -> String {
    let s = time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    s.replace([':', '-'], "").replace('T', "_")
}

fn short_hash(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    hex::encode(out)[0..12].to_string()
}

fn ensure_dir(p: &Path) -> Result<()> {
    fs::create_dir_all(p).with_context(|| format!("create_dir_all failed: {}", p.display()))?;
    Ok(())
}

fn stable_list_generations(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut v: Vec<PathBuf> = Vec::new();
    if !dir.exists() {
        return Ok(v);
    }

    for e in fs::read_dir(dir).with_context(|| format!("read_dir failed: {}", dir.display()))? {
        let p = e?.path();
        if p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("diycad") {
            v.push(p);
        }
    }

    v.sort();
    Ok(v)
}

fn total_bytes(paths: &[PathBuf]) -> u64 {
    let mut s = 0u64;
    for p in paths {
        if let Ok(m) = fs::metadata(p) {
            s = s.saturating_add(m.len());
        }
    }
    s
}

pub struct AutosaveContext {
    pub dir: PathBuf,
    pub policy: RecoveryPolicy,
    pub last_good: Option<PathBuf>,
    pub counter: u64,
}

impl AutosaveContext {
    pub fn new(dir: PathBuf, policy: RecoveryPolicy) -> Self {
        Self {
            dir,
            policy,
            last_good: None,
            counter: 0,
        }
    }
}

pub struct AutosaveResult {
    pub saved: bool,
    pub generation_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

pub fn autosave_if_dirty(
    ctx: &mut AutosaveContext,
    dirty: bool,
    manifest: Manifest,
    document: Document,
    parts: Vec<Part>,
    nest_jobs: Vec<NestJob>,
) -> Result<AutosaveResult> {
    let mut warnings: Vec<String> = Vec::new();

    if !dirty {
        warnings.push(RecoveryReason::AutosaveSkippedNotDirty.as_str().to_string());
        return Ok(AutosaveResult {
            saved: false,
            generation_path: None,
            warnings,
        });
    }

    ensure_dir(&ctx.dir)?;

    let basis = format!(
        "{}|{}|{}",
        manifest.schema_version, document.id, document.name
    );
    let h = short_hash(basis.as_bytes());
    let name = format!("{}_{}_{}.diycad", now_compact(), ctx.counter, h);
    ctx.counter = ctx.counter.saturating_add(1);
    let path = ctx.dir.join(name);

    save_package(
        &path,
        SaveOptions::default(),
        manifest,
        document,
        parts,
        nest_jobs,
        vec![],
    )
    .map_err(|e| anyhow!("{}: {}", RecoveryReason::AutosaveWriteFailed.as_str(), e))?;

    if let Err(e) = prune_generations(ctx) {
        warnings.push(format!(
            "{}: {}",
            RecoveryReason::AutosavePruneFailed.as_str(),
            e
        ));
    }

    Ok(AutosaveResult {
        saved: true,
        generation_path: Some(path),
        warnings,
    })
}

fn prune_generations(ctx: &AutosaveContext) -> Result<()> {
    let mut gens = stable_list_generations(&ctx.dir)?;
    let protected = if ctx.policy.keep_last_good {
        ctx.last_good.clone()
    } else {
        None
    };

    let mut tmp_files: Vec<PathBuf> = Vec::new();
    for e in fs::read_dir(&ctx.dir)? {
        let p = e?.path();
        if p.is_file()
            && p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with(".tmp"))
        {
            tmp_files.push(p);
        }
    }
    tmp_files.sort();
    for p in tmp_files {
        let _ = fs::remove_file(p);
    }

    while gens.len() > ctx.policy.max_generations {
        let candidate = gens.remove(0);
        if protected.as_ref().is_some_and(|x| x == &candidate) {
            gens.push(candidate);
            gens.sort();
            if gens.len() <= 1 {
                break;
            }
            continue;
        }
        let _ = fs::remove_file(candidate);
    }

    loop {
        if total_bytes(&gens) <= ctx.policy.max_total_bytes || gens.is_empty() {
            break;
        }

        let candidate = gens.remove(0);
        if protected.as_ref().is_some_and(|x| x == &candidate) {
            gens.push(candidate);
            gens.sort();
            break;
        }
        let _ = fs::remove_file(candidate);
    }

    Ok(())
}

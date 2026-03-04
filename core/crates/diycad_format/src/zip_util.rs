use std::io::{Read, Seek};

use security::{
    ExternalRefPolicy, Limits, LimitsProfile, PathValidationContext, Sandbox, ZipStats,
};

use crate::ReasonCode;

pub fn map_sec_code(code: security::SecCode) -> ReasonCode {
    match code.as_str() {
        "SEC_PATH_TRAVERSAL_BLOCKED" => ReasonCode::SecZipTraversal,
        "SEC_ABSOLUTE_PATH_BLOCKED" | "SEC_DEVICE_PATH_BLOCKED" => ReasonCode::SecZipAbsolutePath,
        "SEC_INVALID_PATH_CHARS" => ReasonCode::SecZipInvalidEntryName,
        "SEC_PATH_TOO_DEEP" => ReasonCode::SecZipPathTooDeep,
        "SEC_ZIP_LIMIT_EXCEEDED" => ReasonCode::SecZipTotalUncompressedTooLarge,
        "SEC_LIMIT_EXCEEDED" => ReasonCode::SecZipEntryTooLarge,
        _ => ReasonCode::SecZipBadZip,
    }
}

pub fn load_security_defaults() -> anyhow::Result<(Limits, Sandbox)> {
    let limits = Limits::load_from_ssot(LimitsProfile::Default)
        .map_err(|e| anyhow::anyhow!("{}: {}", map_sec_code(e.code).as_str(), e.message))?;
    let sandbox = Sandbox::new(ExternalRefPolicy::Reject);
    Ok((limits, sandbox))
}

pub fn zip_preflight<R: Read + Seek>(
    limits: &Limits,
    sandbox: &Sandbox,
    zip: &mut zip::ZipArchive<R>,
) -> anyhow::Result<ZipStats> {
    let mut entries: u64 = 0;
    let mut total_uncompressed: u64 = 0;
    let mut max_entry: u64 = 0;
    let mut max_depth: u64 = 0;

    for i in 0..zip.len() {
        entries += 1;
        let f = zip.by_index(i).map_err(|e| {
            anyhow::anyhow!(
                "{}: zip read failed: {e}",
                ReasonCode::SecZipBadZip.as_str()
            )
        })?;
        let name = f.name().to_string();
        if f.is_dir() {
            continue;
        }

        let ctx = PathValidationContext {
            max_depth: limits.max_path_depth,
        };
        let norm = sandbox
            .normalize_rel_path(ctx, &name)
            .map_err(|e| anyhow::anyhow!("{}: {}", map_sec_code(e.code).as_str(), e.message))?;
        let depth = norm.as_str().split('/').count() as u64;
        max_depth = max_depth.max(depth);

        let uncomp = f.size();
        max_entry = max_entry.max(uncomp);
        total_uncompressed = total_uncompressed.saturating_add(uncomp);
    }

    let stats = ZipStats {
        entries,
        total_uncompressed_bytes: total_uncompressed,
        max_entry_bytes: max_entry,
        max_path_depth: max_depth,
    };
    limits
        .check_zip(stats)
        .map_err(|e| anyhow::anyhow!("{}: {}", map_sec_code(e.code).as_str(), e.message))?;
    Ok(stats)
}

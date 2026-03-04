use crate::{
    build_content_manifest, ContentManifest, Document, Manifest, NestJob, Part, ReasonCode,
    SaveOptions, Unit,
};
use anyhow::{anyhow, Context, Result};
use security::LimitKind;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

fn now_rfc3339() -> String {
    let fmt = time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&fmt)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn normalize_json_value(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut new = serde_json::Map::new();
            for k in keys {
                let mut vv = map.remove(&k).expect("key exists");
                normalize_json_value(&mut vv);
                new.insert(k, vv);
            }
            *map = new;
        }
        serde_json::Value::Array(arr) => {
            for x in arr {
                normalize_json_value(x);
            }
        }
        _ => {}
    }
}

fn normalize_document(doc: &mut Document) {
    doc.parts_index.sort();
    doc.nest_jobs_index.sort();
}

fn normalize_parts(parts: &mut [Part]) {
    parts.sort_by(|a, b| a.id.cmp(&b.id));
    for p in parts {
        if let Some(tags) = p.tags.as_mut() {
            tags.sort();
            tags.dedup();
        }
    }
}

fn normalize_nest_jobs(nj: &mut [NestJob]) {
    nj.sort_by(|a, b| a.id.cmp(&b.id));
}

fn write_zip_atomic(target: &Path, tmp: &Path) -> Result<()> {
    let f = File::open(tmp).with_context(|| "reopen tmp for sync")?;
    f.sync_all()
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicFsyncFailed.as_str(), e))?;
    drop(f);
    fs::rename(tmp, target)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicRenameFailed.as_str(), e))?;
    Ok(())
}

pub fn save_package(
    path: &Path,
    opt: SaveOptions,
    manifest: Manifest,
    mut document: Document,
    mut parts: Vec<Part>,
    mut nest_jobs: Vec<NestJob>,
    assets: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    if opt.validate_before_save {
        if document.id.is_empty() || document.name.is_empty() {
            return Err(anyhow!(
                "{}: document id/name required",
                ReasonCode::SaveValidateFailed.as_str()
            ));
        }
        if manifest.schema_version < 1 {
            return Err(anyhow!(
                "{}: manifest.schema_version invalid",
                ReasonCode::SaveValidateFailed.as_str()
            ));
        }
        let unit_m = match manifest.unit {
            Unit::Mm => "mm",
            Unit::Inch => "inch",
        };
        let unit_d = match document.unit {
            Unit::Mm => "mm",
            Unit::Inch => "inch",
        };
        if unit_m != unit_d {
            return Err(anyhow!(
                "{}: unit mismatch between manifest and document",
                ReasonCode::SaveValidateFailed.as_str()
            ));
        }
    }

    let (sec_limits, _sec_sandbox) = crate::load_security_defaults()?;

    if opt.normalize_before_save {
        normalize_document(&mut document);
        normalize_parts(&mut parts);
        normalize_nest_jobs(&mut nest_jobs);
    }

    let mut doc_v = serde_json::to_value(&document)?;
    normalize_json_value(&mut doc_v);
    let doc_bytes = serde_json::to_vec_pretty(&doc_v)?;
    sec_limits
        .check_bytes(LimitKind::SingleEntryBytes, doc_bytes.len() as u64)
        .map_err(|e| anyhow!("{}: {}", crate::map_sec_code(e.code).as_str(), e.message))?;

    let mut man = manifest.clone();
    man.updated_at = now_rfc3339();
    if man.entrypoints.document.is_empty() {
        man.entrypoints.document = "document.json".to_string();
    }

    let mut entry_bytes: Vec<(String, Vec<u8>)> = Vec::new();

    let mut part_entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(parts.len());
    for p in &parts {
        let mut v = serde_json::to_value(p)?;
        normalize_json_value(&mut v);
        part_entries.push((
            format!("parts/{}.json", p.id),
            serde_json::to_vec_pretty(&v)?,
        ));
    }
    part_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut nest_entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(nest_jobs.len());
    for j in &nest_jobs {
        let mut v = serde_json::to_value(j)?;
        normalize_json_value(&mut v);
        nest_entries.push((
            format!("nest_jobs/{}.json", j.id),
            serde_json::to_vec_pretty(&v)?,
        ));
    }
    nest_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut assets_entries = Vec::new();
    if opt.include_assets {
        for (p, b) in assets {
            if p.starts_with("assets/") {
                assets_entries.push((p, b));
            }
        }
        assets_entries.sort_by(|a, b| a.0.cmp(&b.0));
    }

    entry_bytes.push(("document.json".to_string(), doc_bytes.clone()));
    entry_bytes.extend(part_entries.iter().cloned());
    entry_bytes.extend(nest_entries.iter().cloned());
    entry_bytes.extend(assets_entries.iter().cloned());

    if opt.write_content_manifest {
        let cm: ContentManifest = build_content_manifest(&entry_bytes);
        man.content_manifest = Some(cm);
    }

    let mut man_v = serde_json::to_value(&man)?;
    normalize_json_value(&mut man_v);
    let man_bytes = serde_json::to_vec_pretty(&man_v)?;
    sec_limits
        .check_bytes(LimitKind::SingleEntryBytes, man_bytes.len() as u64)
        .map_err(|e| anyhow!("{}: {}", crate::map_sec_code(e.code).as_str(), e.message))?;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project.diycad");
    let tmp_path: PathBuf = parent.join(format!(".{}.tmp", fname));

    let tmp_file = File::create(&tmp_path)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicTempCreateFailed.as_str(), e))?;
    let mut zipw = zip::ZipWriter::new(tmp_file);
    let zopt = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zipw.start_file("manifest.json", zopt)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    zipw.write_all(&man_bytes)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;

    zipw.start_file("document.json", zopt)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    zipw.write_all(&doc_bytes)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;

    for (p, b) in part_entries {
        zipw.start_file(p, zopt)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
        zipw.write_all(&b)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    }
    for (p, b) in nest_entries {
        zipw.start_file(p, zopt)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
        zipw.write_all(&b)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    }
    for (p, b) in assets_entries {
        zipw.start_file(p, zopt)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
        zipw.write_all(&b)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    }

    let tmp_file = zipw
        .finish()
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
    tmp_file
        .sync_all()
        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicFsyncFailed.as_str(), e))?;

    if opt.atomic {
        write_zip_atomic(path, &tmp_path)?;
    } else {
        fs::copy(&tmp_path, path)
            .map_err(|e| anyhow!("{}: {}", ReasonCode::IoWriteFailed.as_str(), e))?;
        let _ = fs::remove_file(&tmp_path);
    }

    Ok(())
}

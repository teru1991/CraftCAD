use crate::{
    open_package_file, AppWarning, Document, FailedEntry, Manifest, NestJob, OpenOptions,
    OpenResult, Part, ReasonCode, SalvageActionHint, WarningKind, ZipIndexEntry,
};
use anyhow::{anyhow, Result};
use migration::Registry;
use serde_json::Value;
use std::path::Path;

#[cfg(feature = "test_latest_2")]
const LATEST_SCHEMA_VERSION: i64 = 2;
#[cfg(not(feature = "test_latest_2"))]
const LATEST_SCHEMA_VERSION: i64 = 1;

fn push_warn(
    warnings: &mut Vec<AppWarning>,
    code: ReasonCode,
    path: Option<String>,
    kind: WarningKind,
    msg: impl Into<String>,
) {
    warnings.push(AppWarning {
        code,
        path,
        kind,
        message: msg.into(),
    });
}

fn stable_sort_warnings(w: &mut [AppWarning]) {
    w.sort_by(|a, b| {
        (a.code, a.path.as_deref().unwrap_or(""), &a.kind).cmp(&(
            b.code,
            b.path.as_deref().unwrap_or(""),
            &b.kind,
        ))
    });
}

fn locate_document_path(
    entries: &[ZipIndexEntry],
    manifest: &Option<Manifest>,
    warnings: &mut Vec<AppWarning>,
) -> Option<String> {
    if let Some(m) = manifest {
        let p = m.entrypoints.document.clone();
        if entries.iter().any(|e| e.path == p) {
            return Some(p);
        }
    }
    for e in entries {
        if e.path.ends_with("document.json") {
            push_warn(
                warnings,
                ReasonCode::OpenDocumentLocateHeuristicUsed,
                Some(e.path.clone()),
                WarningKind::Warning,
                "document located by heuristic (manifest missing or invalid)",
            );
            return Some(e.path.clone());
        }
    }
    None
}

fn parse_json<T: serde::de::DeserializeOwned>(bytes: &[u8], _strict: bool) -> Result<T> {
    Ok(serde_json::from_slice(bytes)?)
}

fn validate_pair(_version: i64, manifest: &Value, document: &Value) -> Result<()> {
    let _: Manifest = serde_json::from_value(manifest.clone())?;
    let _: Document = serde_json::from_value(document.clone())?;
    Ok(())
}

pub fn open_package(path: &Path, opt: OpenOptions) -> Result<OpenResult> {
    let mut pkg = open_package_file(path, &opt.limits)?;

    let mut warnings: Vec<AppWarning> = Vec::new();
    let mut salvage_actions: Vec<SalvageActionHint> = Vec::new();
    let mut read_only = false;
    let mut migrate_report = None;

    let mut manifest: Option<Manifest> = match pkg
        .read_entry_bytes("manifest.json", opt.limits.max_entry_uncompressed)?
    {
        None => {
            push_warn(
                &mut warnings,
                ReasonCode::OpenManifestMissing,
                Some("manifest.json".to_string()),
                WarningKind::Warning,
                "manifest missing; continuing with salvage heuristics",
            );
            None
        }
        Some(bytes) => match parse_json::<Manifest>(&bytes, opt.strict_schema) {
            Ok(m) => {
                if m.schema_version > LATEST_SCHEMA_VERSION {
                    if opt.allow_forward_compat_readonly {
                        read_only = true;
                        push_warn(
                            &mut warnings,
                            ReasonCode::OpenSchemaForwardIncompatibleReadonly,
                            Some("manifest.json".to_string()),
                            WarningKind::Warning,
                            format!(
                                "forward schema_version={} > latest={} => open read-only best-effort",
                                m.schema_version, LATEST_SCHEMA_VERSION
                            ),
                        );
                    } else {
                        return Err(anyhow!(
                            "{}: forward incompatible schema_version={} > latest={}",
                            ReasonCode::OpenSchemaForwardIncompatibleReadonly.as_str(),
                            m.schema_version,
                            LATEST_SCHEMA_VERSION
                        ));
                    }
                } else if m.schema_version < (LATEST_SCHEMA_VERSION - 2) {
                    push_warn(
                        &mut warnings,
                        ReasonCode::OpenSchemaTooOldSuggestMigrate,
                        Some("manifest.json".to_string()),
                        WarningKind::Warning,
                        format!(
                            "too old schema_version={} (supported >= {})",
                            m.schema_version,
                            LATEST_SCHEMA_VERSION - 2
                        ),
                    );
                    salvage_actions.push(SalvageActionHint::SuggestMigrateTool);
                }
                Some(m)
            }
            Err(e) => {
                push_warn(
                    &mut warnings,
                    ReasonCode::OpenManifestInvalidJson,
                    Some("manifest.json".to_string()),
                    WarningKind::Warning,
                    format!("manifest invalid: {}", e),
                );
                None
            }
        },
    };

    let doc_path =
        locate_document_path(&pkg.entries, &manifest, &mut warnings).ok_or_else(|| {
            anyhow!(
                "{}: document.json not found",
                ReasonCode::OpenDocumentMissing.as_str()
            )
        })?;
    let doc_bytes = pkg
        .read_entry_bytes(&doc_path, opt.limits.max_entry_uncompressed)?
        .ok_or_else(|| {
            anyhow!(
                "{}: missing {}",
                ReasonCode::OpenDocumentMissing.as_str(),
                doc_path
            )
        })?;
    let document: Document = parse_json::<Document>(&doc_bytes, opt.strict_schema)
        .map_err(|e| anyhow!("{}: {}", ReasonCode::OpenDocumentInvalidJson.as_str(), e))?;

    let mut parts_loaded: Vec<Part> = Vec::new();
    let mut parts_failed: Vec<FailedEntry> = Vec::new();
    let mut nest_jobs_loaded: Vec<NestJob> = Vec::new();
    let mut nest_jobs_failed: Vec<FailedEntry> = Vec::new();

    let assets_count = pkg
        .entries
        .iter()
        .filter(|e| e.path.starts_with("assets/") && !e.path.ends_with('/'))
        .count();

    let parts_total = pkg
        .entries
        .iter()
        .filter(|e| e.path.starts_with("parts/") && e.path.ends_with(".json"))
        .count();
    let part_paths: Vec<String> = pkg
        .entries
        .iter()
        .filter(|e| e.path.starts_with("parts/") && e.path.ends_with(".json"))
        .map(|e| e.path.clone())
        .take(opt.limits.max_parts)
        .collect();
    if parts_total > opt.limits.max_parts {
        read_only = true;
        push_warn(
            &mut warnings,
            ReasonCode::SecZipTooManyEntries,
            Some("parts/".to_string()),
            WarningKind::Warning,
            format!(
                "too many parts; partial load up to {}",
                opt.limits.max_parts
            ),
        );
    }

    for p in part_paths {
        let b = match pkg.read_entry_bytes(&p, opt.limits.max_entry_uncompressed) {
            Ok(Some(x)) => x,
            Ok(None) => continue,
            Err(e) => {
                read_only = true;
                parts_failed.push(FailedEntry {
                    path: p,
                    code: ReasonCode::IoReadFailed,
                    message: e.to_string(),
                });
                continue;
            }
        };
        match parse_json::<Part>(&b, opt.strict_schema) {
            Ok(v) => parts_loaded.push(v),
            Err(e) => {
                read_only = true;
                parts_failed.push(FailedEntry {
                    path: p,
                    code: ReasonCode::OpenPartInvalidJson,
                    message: e.to_string(),
                });
            }
        }
    }

    let nest_jobs_total = pkg
        .entries
        .iter()
        .filter(|e| e.path.starts_with("nest_jobs/") && e.path.ends_with(".json"))
        .count();
    let nj_paths: Vec<String> = pkg
        .entries
        .iter()
        .filter(|e| e.path.starts_with("nest_jobs/") && e.path.ends_with(".json"))
        .map(|e| e.path.clone())
        .take(opt.limits.max_nest_jobs)
        .collect();
    if nest_jobs_total > opt.limits.max_nest_jobs {
        read_only = true;
        push_warn(
            &mut warnings,
            ReasonCode::SecZipTooManyEntries,
            Some("nest_jobs/".to_string()),
            WarningKind::Warning,
            format!(
                "too many nest_jobs; partial load up to {}",
                opt.limits.max_nest_jobs
            ),
        );
    }

    for p in nj_paths {
        let b = match pkg.read_entry_bytes(&p, opt.limits.max_entry_uncompressed) {
            Ok(Some(x)) => x,
            Ok(None) => continue,
            Err(e) => {
                read_only = true;
                nest_jobs_failed.push(FailedEntry {
                    path: p,
                    code: ReasonCode::IoReadFailed,
                    message: e.to_string(),
                });
                continue;
            }
        };
        match parse_json::<NestJob>(&b, opt.strict_schema) {
            Ok(v) => nest_jobs_loaded.push(v),
            Err(e) => {
                read_only = true;
                nest_jobs_failed.push(FailedEntry {
                    path: p,
                    code: ReasonCode::OpenNestJobInvalidJson,
                    message: e.to_string(),
                });
            }
        }
    }

    if let Some(mut m) = manifest.clone() {
        if m.schema_version < LATEST_SCHEMA_VERSION {
            let mut manifest_v = serde_json::to_value(&m)?;
            let mut document_v = serde_json::to_value(&document)?;
            let mut parts_v: Vec<Value> = parts_loaded
                .iter()
                .map(serde_json::to_value)
                .collect::<std::result::Result<_, _>>()?;
            let mut nest_jobs_v: Vec<Value> = nest_jobs_loaded
                .iter()
                .map(serde_json::to_value)
                .collect::<std::result::Result<_, _>>()?;

            let before = (parts_v.len(), nest_jobs_v.len(), assets_count);
            let after = before;

            #[cfg(feature = "test_latest_2")]
            let registry = Registry::new().register(Box::new(crate::migrate_steps::Step1to2));
            #[cfg(not(feature = "test_latest_2"))]
            let registry = Registry::new();

            let mut vin = |version: i64, man: &Value, doc: &Value| {
                let _ = version;
                validate_pair(version, man, doc)
            };
            let mut vout = |version: i64, man: &Value, doc: &Value| {
                let _ = version;
                validate_pair(version, man, doc)
            };

            match registry.migrate_stepwise(
                m.schema_version,
                LATEST_SCHEMA_VERSION,
                &mut manifest_v,
                &mut document_v,
                &mut parts_v,
                &mut nest_jobs_v,
                before,
                after,
                false,
                &mut vin,
                &mut vout,
            ) {
                Ok(rep) => {
                    m = serde_json::from_value(manifest_v)?;
                    manifest = Some(m);
                    migrate_report = Some(rep);
                    push_warn(
                        &mut warnings,
                        ReasonCode::MigrateApplied,
                        Some("manifest.json".to_string()),
                        WarningKind::Warning,
                        "migration applied successfully",
                    );
                }
                Err(e) => {
                    read_only = true;
                    salvage_actions.push(SalvageActionHint::SuggestMigrateTool);
                    push_warn(
                        &mut warnings,
                        ReasonCode::MigrateFailed,
                        Some("manifest.json".to_string()),
                        WarningKind::Warning,
                        format!("migration failed; opened with salvage state: {e}"),
                    );
                }
            }
        }
    }

    if opt.verify_integrity {
        if let Some(m) = &manifest {
            let mut fetch = |p: &str| -> Result<Option<Vec<u8>>> {
                pkg.read_entry_bytes(p, opt.limits.max_entry_uncompressed)
            };
            let (ro, mut w2, mut s2) = crate::verify_content_manifest(m, &mut fetch)?;
            read_only |= ro;
            warnings.append(&mut w2);
            salvage_actions.append(&mut s2);
        } else {
            push_warn(
                &mut warnings,
                ReasonCode::SaveIntegrityManifestMissing,
                Some("manifest.json".to_string()),
                WarningKind::Warning,
                "manifest unavailable; cannot verify content_manifest",
            );
        }
    }

    if !parts_failed.is_empty() || !nest_jobs_failed.is_empty() {
        read_only = true;
        salvage_actions.push(SalvageActionHint::ExportSalvagedParts);
        salvage_actions.push(SalvageActionHint::ResaveAsNewProject);
    }

    parts_failed.sort_by(|a, b| a.path.cmp(&b.path));
    nest_jobs_failed.sort_by(|a, b| a.path.cmp(&b.path));
    stable_sort_warnings(&mut warnings);

    let preferred = [
        SalvageActionHint::ExportSalvagedDocument,
        SalvageActionHint::ExportSalvagedParts,
        SalvageActionHint::GenerateDiagnosticsZip,
        SalvageActionHint::ResaveAsNewProject,
        SalvageActionHint::SuggestMigrateTool,
    ];
    let mut uniq: Vec<SalvageActionHint> = Vec::new();
    for x in preferred {
        if salvage_actions.iter().any(|y| *y == x) && !uniq.contains(&x) {
            uniq.push(x);
        }
    }

    Ok(OpenResult {
        read_only,
        manifest,
        document,
        parts_loaded,
        parts_failed,
        nest_jobs_loaded,
        nest_jobs_failed,
        warnings,
        salvage_actions: uniq,
        migrate_report,
    })
}

use crate::joblog::JobLog;
use crate::oplog::OpLog;
use crate::reason_summary::ReasonSummary;
use crate::retention::RetentionPolicy;
use crate::ssot_fingerprint::SsotFingerprint;
use crate::store::{DiagnosticsStore, StoreIndexEntry};
use crate::SecurityCtx;
use craftcad_dirty_engine::DirtyPlanV1;
use craftcad_estimate_lite::{compute_estimate_lite, estimate_hash_hex};
use craftcad_mfg_hints_lite::{
    compute_fastener_bom_with_hints_lite, fastener_bom_hash_hex, hints_hash_hex,
};
use craftcad_perf::PerfReport;
use craftcad_projection_lite::{project_to_sheet_lite, sheet_hash_hex, Aabb, PartBox, ViewLite};
use craftcad_ssot::SsotV1;
use craftcad_viewpack::build_viewpack_from_ssot;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use zip::write::FileOptions;

static ZIP_TS_FN: OnceLock<fn() -> String> = OnceLock::new();

fn zip_timestamp_compact_utc() -> String {
    if let Some(f) = ZIP_TS_FN.get() {
        return f();
    }
    let ts = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
    ts.replace([':', '-'], "")
}

pub fn set_zip_ts_fn_for_tests(f: fn() -> String) {
    let _ = ZIP_TS_FN.set(f);
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DerivedHashes {
    pub projection_front: String,
    pub projection_top: String,
    pub projection_side: String,
    pub estimate: String,
    pub fastener_bom: String,
    pub mfg_hints: String,
    pub viewpack: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReproEnv {
    pub git_sha: String,
    pub rustc_version: String,
    pub os: String,
    pub app_version: String,
}

#[derive(Clone, Debug)]
pub struct SupportZipReproBundle {
    pub ssot_snapshot_redacted: serde_json::Value,
    pub derived_hashes: DerivedHashes,
    pub dirty_plan: Option<DirtyPlanV1>,
    pub env: ReproEnv,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn detect_git_sha() -> String {
    if let Ok(v) = std::env::var("GITHUB_SHA") {
        if !v.trim().is_empty() {
            return v;
        }
    }
    if let Ok(v) = std::env::var("GIT_SHA") {
        if !v.trim().is_empty() {
            return v;
        }
    }
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_rustc_version() -> String {
    std::process::Command::new("rustc")
        .arg("-V")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn to_projection_part_boxes(ssot: &SsotV1) -> Vec<PartBox> {
    let mut parts = ssot.parts.clone();
    parts.sort_by_key(|p| p.part_id);
    parts
        .into_iter()
        .map(|part| {
            let aabb = match part.manufacturing_outline_2d {
                Some(outline) => Aabb {
                    min_x: outline.min_x.min(outline.max_x),
                    min_y: outline.min_y.min(outline.max_y),
                    min_z: 0.0,
                    max_x: outline.max_x.max(outline.min_x),
                    max_y: outline.max_y.max(outline.min_y),
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
                None => Aabb {
                    min_x: 0.0,
                    min_y: 0.0,
                    min_z: 0.0,
                    max_x: 100.0,
                    max_y: 100.0,
                    max_z: part.thickness_mm.unwrap_or(0.0).max(0.0),
                },
            };
            PartBox {
                part_id: part.part_id,
                aabb,
            }
        })
        .collect()
}

fn derive_hashes_from_ssot(ssot: &SsotV1) -> io::Result<DerivedHashes> {
    let boxes = to_projection_part_boxes(ssot);
    let projection_front = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Front, boxes.clone()));
    let projection_top = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Top, boxes.clone()));
    let projection_side = sheet_hash_hex(&project_to_sheet_lite(ViewLite::Side, boxes));

    let estimate = estimate_hash_hex(&compute_estimate_lite(ssot));

    let fastener_bundle = compute_fastener_bom_with_hints_lite(ssot)
        .map_err(|(code, message)| io::Error::other(format!("{code}: {message}")))?;
    let fastener_bom = fastener_bom_hash_hex(&fastener_bundle.fastener_bom);
    let mfg_hints = hints_hash_hex(&fastener_bundle.mfg_hints);

    let viewpack = build_viewpack_from_ssot(ssot)
        .map_err(|(code, message)| io::Error::other(format!("{code}: {message}")))?;
    let viewpack_bytes = serde_json::to_vec(&viewpack).map_err(io::Error::other)?;

    Ok(DerivedHashes {
        projection_front,
        projection_top,
        projection_side,
        estimate,
        fastener_bom,
        mfg_hints,
        viewpack: sha256_hex(&viewpack_bytes),
    })
}

#[derive(Clone, Debug)]
pub struct ZipResult {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub warnings: Vec<String>,
}

pub struct SupportZipBuilder {
    joblog: Option<JobLog>,
    oplog: Option<OpLog>,
    reason_summary: Option<ReasonSummary>,
    perf: Option<serde_json::Value>,
    project_snapshot: Option<String>,
    consent: Option<security::ConsentState>,
    inputs_copy: Vec<(String, Vec<u8>)>,
    ssot_fingerprint: Option<SsotFingerprint>,
    repro_bundle: Option<SupportZipReproBundle>,
}

impl Default for SupportZipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SupportZipBuilder {
    pub fn new() -> Self {
        Self {
            joblog: None,
            oplog: None,
            reason_summary: None,
            perf: None,
            project_snapshot: None,
            consent: None,
            inputs_copy: Vec::new(),
            ssot_fingerprint: None,
            repro_bundle: None,
        }
    }

    pub fn attach_joblog(mut self, joblog: JobLog) -> Self {
        self.joblog = Some(joblog);
        self
    }

    pub fn attach_oplog(mut self, oplog: OpLog) -> Self {
        self.oplog = Some(oplog);
        self
    }

    pub fn attach_reason_summary(mut self, summary: ReasonSummary) -> Self {
        self.reason_summary = Some(summary);
        self
    }

    pub fn attach_perf(mut self, perf: PerfReport) -> Self {
        self.perf = serde_json::to_value(perf).ok();
        self
    }

    pub fn attach_perf_report(mut self, perf: serde_json::Value) -> Self {
        self.perf = Some(perf);
        self
    }

    pub fn attach_project_snapshot(mut self, snapshot: Option<String>) -> Self {
        self.project_snapshot = snapshot;
        self
    }

    pub fn attach_consent(mut self, consent: security::ConsentState) -> Self {
        self.consent = Some(consent);
        self
    }

    pub fn attach_inputs_copy(mut self, inputs: Vec<(String, Vec<u8>)>) -> Self {
        self.inputs_copy = inputs;
        self
    }

    pub fn attach_ssot_fingerprint(mut self, fingerprint: SsotFingerprint) -> Self {
        self.ssot_fingerprint = Some(fingerprint);
        self
    }

    pub fn support_zip_add_repro_bundle(
        mut self,
        ssot: &SsotV1,
        hashes: Option<DerivedHashes>,
        dirty_plan_opt: Option<DirtyPlanV1>,
        app_version: Option<&str>,
    ) -> io::Result<Self> {
        let sec = SecurityCtx::load_default()?;
        let canonical_ssot = ssot.clone().canonicalize();
        let ssot_json = serde_json::to_value(canonical_ssot).map_err(io::Error::other)?;
        let ssot_snapshot_redacted = sec.redactor.redact_json(&ssot_json);

        let derived_hashes = match hashes {
            Some(h) => h,
            None => derive_hashes_from_ssot(ssot)?,
        };

        let env = ReproEnv {
            git_sha: detect_git_sha(),
            rustc_version: detect_rustc_version(),
            os: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            app_version: app_version
                .map(ToString::to_string)
                .or_else(|| std::env::var("CRAFTCAD_APP_VERSION").ok())
                .unwrap_or_else(|| "unknown".to_string()),
        };

        self.repro_bundle = Some(SupportZipReproBundle {
            ssot_snapshot_redacted,
            derived_hashes,
            dirty_plan: dirty_plan_opt,
            env,
        });
        Ok(self)
    }

    pub fn build(self, path: impl AsRef<Path>) -> io::Result<PathBuf> {
        let path = path.as_ref();
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let mut result = self.build_in_dir(parent)?;
        if result.path != path {
            if let Err(_e) = fs::rename(&result.path, path) {
                fs::copy(&result.path, path)?;
                let _ = fs::remove_file(&result.path);
            }
            result.path = path.to_path_buf();
            result.size_bytes = fs::metadata(path)?.len();
            result.sha256 = sha256_file(path)?;
        }
        Ok(result.path)
    }

    pub fn build_in_dir(self, out_dir: &Path) -> io::Result<ZipResult> {
        fs::create_dir_all(out_dir)?;
        let path = if out_dir.file_name().map(|n| n == "items").unwrap_or(false) {
            out_dir.join(format!("support-{}.zip", zip_timestamp_compact_utc()))
        } else {
            out_dir.join("support.zip")
        };
        let tmp = path.with_extension("tmp");

        let sec = SecurityCtx::load_default()?;
        let consent_out = sec.consent_store.load();
        let effective_consent = self.consent.unwrap_or(consent_out.state);

        let file = fs::File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let SupportZipBuilder {
            joblog,
            oplog,
            reason_summary,
            perf,
            project_snapshot,
            consent: _,
            inputs_copy,
            ssot_fingerprint,
            repro_bundle,
        } = self;

        let fingerprint = ssot_fingerprint.unwrap_or_else(SsotFingerprint::empty);

        if let Some(joblog) = joblog {
            zip.start_file("joblog.json", opts)?;
            let v = sec
                .redactor
                .redact_json(&serde_json::to_value(joblog).map_err(io::Error::other)?);
            let payload = serde_json::to_vec_pretty(&v).map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;
        }

        let summary = reason_summary.unwrap_or(ReasonSummary {
            top_reasons: Vec::new(),
            suggested_actions: Vec::new(),
        });
        zip.start_file("reason_summary.json", opts)?;
        let summary_payload = serde_json::to_vec_pretty(&summary).map_err(io::Error::other)?;
        sec.limits
            .check_bytes(
                security::LimitKind::SingleEntryBytes,
                summary_payload.len() as u64,
            )
            .map_err(|e| io::Error::other(e.message.to_string()))?;
        zip.write_all(&summary_payload)?;

        if let Some(oplog) = oplog {
            zip.start_file("oplog.json", opts)?;
            let payload = serde_json::to_vec_pretty(&oplog).map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;
        }

        if let Some(perf) = perf {
            zip.start_file("perf_report.json", opts)?;
            let payload = serde_json::to_vec_pretty(&perf).map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;
        }

        zip.start_file("consent.json", opts)?;
        zip.write_all(&serde_json::to_vec_pretty(&effective_consent).map_err(io::Error::other)?)?;

        if effective_consent.diagnostics_include_project {
            if let Some(snapshot) = project_snapshot {
                zip.start_file("project_snapshot.diycad", opts)?;
                sec.limits
                    .check_bytes(security::LimitKind::SingleEntryBytes, snapshot.len() as u64)
                    .map_err(|e| io::Error::other(e.message.to_string()))?;
                zip.write_all(snapshot.as_bytes())?;
            }
        }

        if effective_consent.diagnostics_include_inputs_copy {
            for (name, bytes) in inputs_copy {
                sec.limits
                    .check_bytes(security::LimitKind::SingleEntryBytes, bytes.len() as u64)
                    .map_err(|e| io::Error::other(e.message.to_string()))?;
                let safe_name = sec
                    .sandbox
                    .normalize_rel_path(
                        security::PathValidationContext {
                            max_depth: sec.limits.max_path_depth,
                        },
                        &name,
                    )
                    .map_err(|e| io::Error::other(e.message.to_string()))?;
                zip.start_file(format!("inputs/{}", safe_name.as_str()), opts)?;
                zip.write_all(&bytes)?;
            }
        }

        zip.start_file("ssot_fingerprint.json", opts)?;
        zip.write_all(&serde_json::to_vec_pretty(&fingerprint).map_err(io::Error::other)?)?;

        if let Some(repro) = repro_bundle {
            zip.start_file("repro/ssot_snapshot.json", opts)?;
            let payload = serde_json::to_vec_pretty(&repro.ssot_snapshot_redacted)
                .map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;

            zip.start_file("repro/derived_hashes.json", opts)?;
            let payload =
                serde_json::to_vec_pretty(&repro.derived_hashes).map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;

            if let Some(dirty_plan) = repro.dirty_plan {
                zip.start_file("repro/dirty_plan.json", opts)?;
                let payload = serde_json::to_vec_pretty(&dirty_plan).map_err(io::Error::other)?;
                sec.limits
                    .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                    .map_err(|e| io::Error::other(e.message.to_string()))?;
                zip.write_all(&payload)?;
            }

            zip.start_file("repro/env.json", opts)?;
            let payload = serde_json::to_vec_pretty(&repro.env).map_err(io::Error::other)?;
            sec.limits
                .check_bytes(security::LimitKind::SingleEntryBytes, payload.len() as u64)
                .map_err(|e| io::Error::other(e.message.to_string()))?;
            zip.write_all(&payload)?;
        }

        zip.finish()?;
        fs::rename(&tmp, &path)?;
        let size_bytes = fs::metadata(&path)?.len();
        sec.limits
            .check_bytes(security::LimitKind::SupportZipBytes, size_bytes)
            .map_err(|e| io::Error::other(e.message.to_string()))?;
        let sha256 = sha256_file(&path)?;

        let mut warnings = fingerprint.warnings;
        for w in consent_out.warnings {
            warnings.push(w.code.as_str().to_string());
        }

        Ok(ZipResult {
            path,
            sha256,
            size_bytes,
            warnings,
        })
    }

    pub fn build_into_store(
        mut self,
        store: &DiagnosticsStore,
        repo_root: &Path,
        policy: &RetentionPolicy,
    ) -> io::Result<ZipResult> {
        let _ = store.cleanup(policy);
        if self.ssot_fingerprint.is_none() {
            self.ssot_fingerprint = Some(SsotFingerprint::compute(repo_root));
        }

        let (item_id, item_dir) = store.create_item_dir()?;
        let mut built = self.build_in_dir(&item_dir)?;
        let final_zip = item_dir.join("support.zip");
        if built.path != final_zip {
            if let Err(_e) = std::fs::rename(&built.path, &final_zip) {
                std::fs::copy(&built.path, &final_zip)?;
                let _ = std::fs::remove_file(&built.path);
            }
            built.path = final_zip;
            built.size_bytes = fs::metadata(&built.path)?.len();
            built.sha256 = sha256_file(&built.path)?;
        }

        let entry = StoreIndexEntry {
            id: item_id.clone(),
            created_at: time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string()),
            rel_dir: format!("items/{item_id}"),
            zip_rel_path: Some(format!("items/{item_id}/support.zip")),
            size_bytes: Some(built.size_bytes),
        };
        let _ = store.append_index(&entry);

        Ok(built)
    }
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let data = fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(hex::encode(h.finalize()))
}

use crate::joblog::JobLog;
use crate::oplog::OpLog;
use crate::reason_summary::ReasonSummary;
use crate::retention::RetentionPolicy;
use crate::ssot_fingerprint::SsotFingerprint;
use crate::store::{DiagnosticsStore, StoreIndexEntry};
use craftcad_perf::PerfReport;
use craftcad_security::{redact_json, ConsentState};
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
    consent: Option<ConsentState>,
    ssot_fingerprint: Option<SsotFingerprint>,
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
            ssot_fingerprint: None,
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

    pub fn attach_consent(mut self, consent: ConsentState) -> Self {
        self.consent = Some(consent);
        self
    }

    pub fn attach_ssot_fingerprint(mut self, fingerprint: SsotFingerprint) -> Self {
        self.ssot_fingerprint = Some(fingerprint);
        self
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
        let file = fs::File::create(&tmp)?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let include_project = self
            .consent
            .as_ref()
            .map(|c| c.support_zip_include_project)
            .unwrap_or(false);

        let SupportZipBuilder {
            joblog,
            oplog,
            reason_summary,
            perf,
            project_snapshot,
            consent,
            ssot_fingerprint,
        } = self;

        let fingerprint = ssot_fingerprint.unwrap_or_else(SsotFingerprint::empty);

        if let Some(joblog) = joblog {
            zip.start_file("joblog.json", opts)?;
            let v = redact_json(serde_json::to_value(joblog).map_err(io::Error::other)?);
            zip.write_all(&serde_json::to_vec_pretty(&v).map_err(io::Error::other)?)?;
        }

        let summary = reason_summary.unwrap_or(ReasonSummary {
            top_reasons: Vec::new(),
            suggested_actions: Vec::new(),
        });
        zip.start_file("reason_summary.json", opts)?;
        zip.write_all(&serde_json::to_vec_pretty(&summary).map_err(io::Error::other)?)?;

        if let Some(oplog) = oplog {
            zip.start_file("oplog.json", opts)?;
            zip.write_all(&serde_json::to_vec_pretty(&oplog).map_err(io::Error::other)?)?;
        }

        if let Some(perf) = perf {
            zip.start_file("perf_report.json", opts)?;
            zip.write_all(&serde_json::to_vec_pretty(&perf).map_err(io::Error::other)?)?;
        }

        if let Some(consent) = consent {
            zip.start_file("consent.json", opts)?;
            zip.write_all(&serde_json::to_vec_pretty(&consent).map_err(io::Error::other)?)?;
        }
        if include_project {
            if let Some(snapshot) = project_snapshot {
                zip.start_file("project_snapshot.diycad", opts)?;
                zip.write_all(snapshot.as_bytes())?;
            }
        }

        zip.start_file("ssot_fingerprint.json", opts)?;
        zip.write_all(&serde_json::to_vec_pretty(&fingerprint).map_err(io::Error::other)?)?;

        zip.finish()?;
        fs::rename(&tmp, &path)?;
        let size_bytes = fs::metadata(&path)?.len();
        let sha256 = sha256_file(&path)?;

        Ok(ZipResult {
            path,
            sha256,
            size_bytes,
            warnings: fingerprint.warnings,
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

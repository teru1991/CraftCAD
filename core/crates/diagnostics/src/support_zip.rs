use crate::joblog::JobLog;
use crate::oplog::OpLog;
use crate::reason_summary::ReasonSummary;
use crate::reasons::diag_codes;
use crate::security_iface::{ConsentProvider, Limits, Redactor};
use crate::ssot_fingerprint::SsotFingerprint;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use time::OffsetDateTime;
use uuid::Uuid;
use zip::write::FileOptions;

#[derive(Clone, Debug)]
pub struct ZipWarnings {
    pub warnings: Vec<String>,
}

impl ZipWarnings {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn push(&mut self, code: &str) {
        if !self.warnings.iter().any(|w| w == code) {
            self.warnings.push(code.to_string());
            self.warnings.sort();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ZipResult {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub warnings: ZipWarnings,
}

pub struct SupportZipBuilder<'a> {
    out_dir: PathBuf,
    #[allow(dead_code)]
    limits: Limits,
    redactor: &'a dyn Redactor,
    consent: &'a dyn ConsentProvider,

    job_id: String,
    joblog: Option<JobLog>,
    oplog: Option<OpLog>,
    reason_summary: Option<ReasonSummary>,
    perf_report_json: Option<Value>,
    ssot_fingerprint: Option<SsotFingerprint>,

    project_snapshot_path: Option<PathBuf>,
    inputs_copy: Vec<(String, PathBuf)>,

    warnings: ZipWarnings,
}

impl<'a> SupportZipBuilder<'a> {
    pub fn new(
        out_dir: impl AsRef<Path>,
        limits: Limits,
        redactor: &'a dyn Redactor,
        consent: &'a dyn ConsentProvider,
    ) -> io::Result<Self> {
        let out_dir = out_dir.as_ref().to_path_buf();
        fs::create_dir_all(&out_dir)?;
        let job_id = Uuid::new_v4().to_string();
        Ok(Self {
            out_dir,
            limits,
            redactor,
            consent,
            job_id,
            joblog: None,
            oplog: None,
            reason_summary: None,
            perf_report_json: None,
            ssot_fingerprint: None,
            project_snapshot_path: None,
            inputs_copy: Vec::new(),
            warnings: ZipWarnings::new(),
        })
    }

    pub fn job_id(&self) -> &str {
        &self.job_id
    }

    pub fn attach_joblog(&mut self, joblog: JobLog) -> &mut Self {
        self.joblog = Some(joblog);
        self
    }

    pub fn attach_oplog(&mut self, oplog: OpLog) -> &mut Self {
        self.oplog = Some(oplog);
        self
    }

    pub fn attach_reason_summary(&mut self, summary: ReasonSummary) -> &mut Self {
        self.reason_summary = Some(summary);
        self
    }

    pub fn attach_perf_report(&mut self, perf_report_json: Value) -> &mut Self {
        self.perf_report_json = Some(perf_report_json);
        self
    }

    pub fn attach_ssot_fingerprint(&mut self, fp: SsotFingerprint) -> &mut Self {
        self.ssot_fingerprint = Some(fp);
        self
    }

    pub fn optionally_attach_project_snapshot(
        &mut self,
        snapshot_path: impl AsRef<Path>,
    ) -> &mut Self {
        if self.consent.include_project_snapshot() {
            self.project_snapshot_path = Some(snapshot_path.as_ref().to_path_buf());
        }
        self
    }

    pub fn optionally_attach_input_copy(
        &mut self,
        zip_name: &str,
        src_path: impl AsRef<Path>,
    ) -> &mut Self {
        if self.consent.include_inputs_copy() {
            let zn = self.redactor.redact_str(zip_name);
            let zn = sanitize_zip_path(&zn, 128);
            self.inputs_copy.push((zn, src_path.as_ref().to_path_buf()));
            self.inputs_copy.sort_by(|a, b| a.0.cmp(&b.0));
        }
        self
    }

    pub fn build(mut self) -> io::Result<ZipResult> {
        // We never crash if attachments are missing; we emit placeholders + warnings.
        let has_joblog = self.joblog.is_some();
        let has_reason = self.reason_summary.is_some();
        let has_fp = self.ssot_fingerprint.is_some();

        let ts = OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "19700101T000000Z".to_string());
        let ts = ts.replace(':', "").replace('-', "");
        let file_name = format!("support-{}-{}.zip", self.job_id, ts);
        let out_path = self.out_dir.join(file_name);

        let mut tmp = NamedTempFile::new_in(&self.out_dir)?;
        {
            let mut zw = zip::ZipWriter::new(&mut tmp);
            let opt = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o644);

            let mut total_added: u64 = 0;
            let mut marker_written = false;

            if has_joblog {
                let bytes = serde_json::to_vec_pretty(self.joblog.as_ref().expect("checked"))
                    .unwrap_or_else(|_| b"{}".to_vec());
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "joblog.json",
                    &bytes,
                    &mut total_added,
                    &mut marker_written,
                )?;
            } else {
                self.warnings.push(diag_codes::DIAG_INTERNAL_ERROR);
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "joblog.json",
                    b"{}",
                    &mut total_added,
                    &mut marker_written,
                )?;
            }

            if has_reason {
                let bytes =
                    serde_json::to_vec_pretty(self.reason_summary.as_ref().expect("checked"))
                        .unwrap_or_else(|_| b"{}".to_vec());
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "reason_summary.json",
                    &bytes,
                    &mut total_added,
                    &mut marker_written,
                )?;
            } else {
                self.warnings.push(diag_codes::DIAG_INTERNAL_ERROR);
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "reason_summary.json",
                    b"{}",
                    &mut total_added,
                    &mut marker_written,
                )?;
            }

            if has_fp {
                let bytes =
                    serde_json::to_vec_pretty(self.ssot_fingerprint.as_ref().expect("checked"))
                        .unwrap_or_else(|_| b"{}".to_vec());
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "ssot_fingerprint.json",
                    &bytes,
                    &mut total_added,
                    &mut marker_written,
                )?;
            } else {
                self.warnings
                    .push(diag_codes::DIAG_SSOT_FINGERPRINT_PARTIAL);
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "ssot_fingerprint.json",
                    b"{\"items\":[],\"warnings\":[\"missing\"]}",
                    &mut total_added,
                    &mut marker_written,
                )?;
            }

            if let Some(pr) = &self.perf_report_json {
                let bytes = serde_json::to_vec_pretty(pr).unwrap_or_else(|_| b"{}".to_vec());
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "perf_report.json",
                    &bytes,
                    &mut total_added,
                    &mut marker_written,
                )?;
            }

            if let Some(op) = &self.oplog {
                let bytes = serde_json::to_vec_pretty(op).unwrap_or_else(|_| b"{}".to_vec());
                self.write_json_entry(
                    &mut zw,
                    opt,
                    "oplog.json",
                    &bytes,
                    &mut total_added,
                    &mut marker_written,
                )?;
            }

            if self.consent.include_project_snapshot() {
                if let Some(p) = self.project_snapshot_path.clone() {
                    self.write_file_path(
                        &mut zw,
                        opt,
                        "project_snapshot.diycad",
                        &p,
                        &mut total_added,
                        &mut marker_written,
                    )?;
                }
            }

            if self.consent.include_inputs_copy() {
                for (zip_name, src) in self.inputs_copy.clone() {
                    let name = format!("inputs/{}", zip_name);
                    self.write_file_path(
                        &mut zw,
                        opt,
                        &name,
                        &src,
                        &mut total_added,
                        &mut marker_written,
                    )?;
                }
            }

            zw.finish()?;
        }

        tmp.as_file_mut().sync_all()?;
        let mut tmp_file = tmp.reopen()?;
        let (sha256, size_bytes) = compute_sha256_and_size(&mut tmp_file)?;
        tmp_file.sync_all()?;
        let _ = tmp_file;

        tmp.persist(&out_path).map_err(|e| e.error)?;

        Ok(ZipResult {
            path: out_path,
            sha256,
            size_bytes,
            warnings: self.warnings,
        })
    }

    fn max_total_bytes(&self) -> u64 {
        64 * 1024 * 1024
    }

    fn max_single_file_bytes(&self) -> u64 {
        16 * 1024 * 1024
    }

    fn write_json_entry(
        &mut self,
        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
        opt: FileOptions,
        name: &str,
        bytes: &[u8],
        total_added: &mut u64,
        marker_written: &mut bool,
    ) -> io::Result<()> {
        let name = sanitize_zip_path(name, 256);
        let single = bytes.len() as u64;
        if single > self.max_single_file_bytes() {
            self.warnings.push(diag_codes::DIAG_ZIP_TRUNCATED);
            let placeholder = b"{\"_excluded\":\"single_file_too_large\"}";
            zw.start_file(name, opt)?;
            zw.write_all(placeholder)?;
            *total_added += placeholder.len() as u64;
            self.ensure_marker(zw, opt, total_added, marker_written)?;
            return Ok(());
        }
        if total_added.saturating_add(single) > self.max_total_bytes() {
            self.warnings.push(diag_codes::DIAG_ZIP_TRUNCATED);
            self.ensure_marker(zw, opt, total_added, marker_written)?;
            return Ok(());
        }
        zw.start_file(name, opt)?;
        zw.write_all(bytes)?;
        *total_added += single;
        Ok(())
    }

    fn write_file_path(
        &mut self,
        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
        opt: FileOptions,
        zip_entry_name: &str,
        src_path: &Path,
        total_added: &mut u64,
        marker_written: &mut bool,
    ) -> io::Result<()> {
        let name = sanitize_zip_path(zip_entry_name, 256);
        let size = fs::metadata(src_path)?.len();

        if size > self.max_single_file_bytes() {
            self.warnings.push(diag_codes::DIAG_ZIP_TRUNCATED);
            let placeholder = b"{\"_excluded\":\"single_file_too_large\"}";
            zw.start_file(name, opt)?;
            zw.write_all(placeholder)?;
            *total_added += placeholder.len() as u64;
            self.ensure_marker(zw, opt, total_added, marker_written)?;
            return Ok(());
        }
        if total_added.saturating_add(size) > self.max_total_bytes() {
            self.warnings.push(diag_codes::DIAG_ZIP_TRUNCATED);
            self.ensure_marker(zw, opt, total_added, marker_written)?;
            return Ok(());
        }

        let mut f = File::open(src_path)?;
        zw.start_file(name, opt)?;
        io::copy(&mut f, zw)?;
        *total_added += size;
        Ok(())
    }

    fn ensure_marker(
        &self,
        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
        opt: FileOptions,
        total_added: &mut u64,
        marker_written: &mut bool,
    ) -> io::Result<()> {
        if !*marker_written {
            zw.start_file("zip_truncated.marker", opt)?;
            zw.write_all(b"true")?;
            *total_added += 4;
            *marker_written = true;
        }
        Ok(())
    }
}

fn sanitize_zip_path(s: &str, max_len: usize) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        let safe = match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' | '/' => ch,
            _ => '_',
        };
        out.push(safe);
        if out.len() >= max_len {
            break;
        }
    }
    while out.starts_with('/') {
        out.remove(0);
    }
    out = out.replace("..", "__").replace('\\', "_");
    if out.is_empty() {
        "unnamed".to_string()
    } else {
        out
    }
}

fn compute_sha256_and_size(f: &mut File) -> io::Result<(String, u64)> {
    f.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    let mut total: u64 = 0;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        hasher.update(&buf[..n]);
    }
    f.seek(SeekFrom::Start(0))?;
    Ok((hex::encode(hasher.finalize()), total))
}

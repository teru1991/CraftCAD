use crate::joblog::JobLog;
use craftcad_perf::PerfReport;
use craftcad_security::{redact_json, ConsentState};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

pub struct SupportZipBuilder {
    joblog: Option<JobLog>,
    perf: Option<PerfReport>,
    project_snapshot: Option<String>,
    consent: Option<ConsentState>,
}

impl SupportZipBuilder {
    pub fn new() -> Self {
        Self {
            joblog: None,
            perf: None,
            project_snapshot: None,
            consent: None,
        }
    }

    pub fn attach_joblog(mut self, joblog: JobLog) -> Self {
        self.joblog = Some(joblog);
        self
    }

    pub fn attach_perf(mut self, perf: PerfReport) -> Self {
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

    pub fn build(self, path: impl AsRef<Path>) -> Result<PathBuf, String> {
        let path = path.as_ref();
        let tmp = path.with_extension("tmp");
        let file = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if let Some(joblog) = self.joblog {
            zip.start_file("joblog.json", opts)
                .map_err(|e| e.to_string())?;
            let v = redact_json(serde_json::to_value(joblog).map_err(|e| e.to_string())?);
            zip.write_all(&serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        if let Some(perf) = self.perf {
            zip.start_file("perf_report.json", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(&serde_json::to_vec_pretty(&perf).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        let include_project = self
            .consent
            .as_ref()
            .map(|c| c.support_zip_include_project)
            .unwrap_or(false);

        if let Some(consent) = self.consent {
            zip.start_file("consent.json", opts)
                .map_err(|e| e.to_string())?;
            zip.write_all(&serde_json::to_vec_pretty(&consent).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        if include_project {
            if let Some(snapshot) = self.project_snapshot {
                zip.start_file("project_snapshot.diycad", opts)
                    .map_err(|e| e.to_string())?;
                zip.write_all(snapshot.as_bytes())
                    .map_err(|e| e.to_string())?;
            }
        }

        zip.finish().map_err(|e| e.to_string())?;
        fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        Ok(path.to_path_buf())
    }
}

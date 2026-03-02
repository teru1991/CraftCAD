#![forbid(unsafe_code)]

pub mod approx;
pub mod model;
pub mod normalize;
pub mod options;
pub mod path_opt;
pub mod postprocess;
pub mod preflight;
pub mod reasons;
pub mod report;

pub use postprocess::{run_shared_export_pipeline, run_shared_import_pipeline};

use model::InternalModel;
use options::{ExportOptions, ImportOptions};
use reasons::{AppError, AppResult, ReasonCode};

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub model: InternalModel,
    pub warnings: Vec<AppError>,
    pub report: report::IoReport,
}

#[derive(Debug, Clone)]
pub struct ExportResult {
    pub bytes: Vec<u8>,
    pub warnings: Vec<AppError>,
    pub report: report::IoReport,
}

pub trait Importer: Send + Sync {
    fn format_id(&self) -> &'static str;
    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult>;
}

pub trait Exporter: Send + Sync {
    fn format_id(&self) -> &'static str;
    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult>;
}

pub struct IoEngine {
    importers: std::collections::BTreeMap<String, Box<dyn Importer>>,
    exporters: std::collections::BTreeMap<String, Box<dyn Exporter>>,
}

impl IoEngine {
    pub fn new() -> Self {
        Self {
            importers: std::collections::BTreeMap::new(),
            exporters: std::collections::BTreeMap::new(),
        }
    }

    pub fn register_importer(mut self, imp: Box<dyn Importer>) -> Self {
        self.importers.insert(imp.format_id().to_string(), imp);
        self
    }

    pub fn register_exporter(mut self, exp: Box<dyn Exporter>) -> Self {
        self.exporters.insert(exp.format_id().to_string(), exp);
        self
    }

    pub fn has_importer(&self, format_id: &str) -> bool {
        self.importers.contains_key(format_id)
    }

    pub fn has_exporter(&self, format_id: &str) -> bool {
        self.exporters.contains_key(format_id)
    }

    pub fn import(
        &self,
        format: &str,
        bytes: &[u8],
        opts: &ImportOptions,
    ) -> AppResult<ImportResult> {
        let imp = self.importers.get(format).ok_or_else(|| {
            AppError::new(ReasonCode::IO_FORMAT_NOT_REGISTERED, "importer not found")
                .with_context("format_id", format.to_string())
                .fatal()
        })?;

        preflight::preflight_bytes(format, bytes, opts)?;

        let mut res = imp.import_bytes(bytes, opts)?;
        run_shared_import_pipeline(&mut res.model, opts, &mut res.warnings);

        res.report.format = format.to_string();
        res.report.entities_out = res.model.entities.len();
        res.report.texts_out = res.model.texts.len();
        res.report.determinism_tag = opts.determinism_tag();
        Ok(res)
    }

    pub fn export(
        &self,
        format: &str,
        model: &InternalModel,
        opts: &ExportOptions,
    ) -> AppResult<ExportResult> {
        let exp = self.exporters.get(format).ok_or_else(|| {
            AppError::new(ReasonCode::IO_FORMAT_NOT_REGISTERED, "exporter not found")
                .with_context("format_id", format.to_string())
                .fatal()
        })?;

        let mut tmp = model.clone();
        let mut pipeline_warnings: Vec<AppError> = Vec::new();
        run_shared_export_pipeline(&mut tmp, opts, &mut pipeline_warnings);

        let mut res = exp.export_bytes(&tmp, opts)?;
        res.warnings.extend(pipeline_warnings);
        res.report.format = format.to_string();
        res.report.entities_in = model.entities.len();
        res.report.texts_in = model.texts.len();
        res.report.entities_out = tmp.entities.len();
        res.report.texts_out = tmp.texts.len();
        res.report.determinism_tag = opts.determinism_tag();
        Ok(res)
    }
}

impl Default for IoEngine {
    fn default() -> Self {
        Self::new()
    }
}

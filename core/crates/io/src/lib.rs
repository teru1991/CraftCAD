// IMPORTANT:
// - Determinism: sorting/rounding/epsilon MUST be stable and SSOT-driven.
// - No panics on untrusted inputs. Return ReasonCode warnings/errors.
// - Best-effort MUST attach ReasonCode + context (epsilon, segments, feature).
// - CI gates rely on normalize() as the canonical comparison baseline.

pub mod model;
pub mod normalize;
pub mod options;
pub mod pipeline;
pub mod preflight;
pub mod report;

use craftcad_errors::AppResult;
use report::IoReport;

pub struct ImportResult {
    pub model: model::InternalModel,
    pub warnings: Vec<craftcad_errors::AppError>,
    pub report: IoReport,
}

pub struct ExportResult {
    pub bytes: Vec<u8>,
    pub warnings: Vec<craftcad_errors::AppError>,
    pub report: IoReport,
}

pub trait Importer {
    fn format_id(&self) -> &'static str;
    fn import_bytes(&self, bytes: &[u8], opts: &options::ImportOptions) -> AppResult<ImportResult>;
}

pub trait Exporter {
    fn format_id(&self) -> &'static str;
    fn export_bytes(
        &self,
        model: &model::InternalModel,
        opts: &options::ExportOptions,
    ) -> AppResult<ExportResult>;
}

pub struct IoEngine {
    importers: Vec<Box<dyn Importer + Send + Sync>>,
    exporters: Vec<Box<dyn Exporter + Send + Sync>>,
}

impl IoEngine {
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
            exporters: Vec::new(),
        }
    }

    pub fn register_importer(mut self, importer: impl Importer + Send + Sync + 'static) -> Self {
        self.importers.push(Box::new(importer));
        self
    }

    pub fn register_exporter(mut self, exporter: impl Exporter + Send + Sync + 'static) -> Self {
        self.exporters.push(Box::new(exporter));
        self
    }

    pub fn importer(&self, format_id: &str) -> Option<&(dyn Importer + Send + Sync)> {
        self.importers
            .iter()
            .find(|importer| importer.format_id() == format_id)
            .map(std::ops::Deref::deref)
    }

    pub fn exporter(&self, format_id: &str) -> Option<&(dyn Exporter + Send + Sync)> {
        self.exporters
            .iter()
            .find(|exporter| exporter.format_id() == format_id)
            .map(std::ops::Deref::deref)
    }
}

impl Default for IoEngine {
    fn default() -> Self {
        Self::new()
    }
}

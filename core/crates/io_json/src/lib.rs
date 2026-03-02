#![forbid(unsafe_code)]

mod export;
mod import;
mod schema;

use craftcad_io::model::InternalModel;
use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::reasons::AppResult;
use craftcad_io::{ExportResult, Exporter, ImportResult, Importer};

pub struct JsonIo;

impl JsonIo {
    pub fn new() -> Self {
        Self
    }
}

impl Importer for JsonIo {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult> {
        let (model, mut warnings, mut report) = import::import_json(bytes, opts)?;
        report.format = "json".to_string();
        report.entities_in = model.entities.len();
        report.texts_in = model.texts.len();

        Ok(ImportResult {
            model,
            warnings: warnings.drain(..).collect(),
            report,
        })
    }
}

impl Exporter for JsonIo {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult> {
        let (bytes, mut warnings, mut report) = export::export_json(model, opts)?;
        report.format = "json".to_string();
        Ok(ExportResult {
            bytes,
            warnings: warnings.drain(..).collect(),
            report,
        })
    }
}

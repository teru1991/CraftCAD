#![forbid(unsafe_code)]

mod export;
mod import;
mod mapping;
mod parse;

use craftcad_io::model::InternalModel;
use craftcad_io::options::{ExportOptions, ImportOptions};
use craftcad_io::reasons::AppResult;
use craftcad_io::{ExportResult, Exporter, ImportResult, Importer};

pub struct SvgIo;

impl SvgIo {
    pub fn new() -> Self {
        Self
    }
}

impl Importer for SvgIo {
    fn format_id(&self) -> &'static str {
        "svg"
    }

    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult> {
        let (model, warnings, report) = import::import_svg(bytes, opts)?;
        Ok(ImportResult {
            model,
            warnings,
            report,
        })
    }
}

impl Exporter for SvgIo {
    fn format_id(&self) -> &'static str {
        "svg"
    }

    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult> {
        let (bytes, warnings, report) = export::export_svg(model, opts)?;
        Ok(ExportResult {
            bytes,
            warnings,
            report,
        })
    }
}

use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use craftcad_io::options::ImportOptions;
use craftcad_io::pipeline::finalize_import;
use craftcad_io::report::IoReport;
use craftcad_io::{ImportResult, Importer};

pub struct JsonImporter;

impl Importer for JsonImporter {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult> {
        crate::preflight::run(bytes, opts)?;
        let model = crate::mapping::parse_model(bytes).map_err(|e| {
            AppError::new(
                ReasonCode::new("IO_LIMIT_016"),
                Severity::Error,
                format!("Failed to parse JSON model: {e}"),
            )
        })?;
        let report = IoReport {
            format: self.format_id().to_string(),
            entities_in: model.entities.len(),
            entities_out: model.entities.len(),
            ..IoReport::default()
        };
        Ok(finalize_import(model, opts, report, Vec::new()))
    }
}

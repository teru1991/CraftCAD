use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
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
                ReasonCode::IoNormalizeRounded,
                format!("Failed to parse JSON model: {e}"),
            )
        })?;
        let mut report = IoReport::new(self.format_id());
        report.entities_in = model.entities.len();
        report.entities_out = model.entities.len();
        Ok(ImportResult {
            model,
            warnings: Vec::new(),
            report,
        })
    }
}

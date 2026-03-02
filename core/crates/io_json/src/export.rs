use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use craftcad_io::{ExportResult, Exporter};

pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn format_id(&self) -> &'static str {
        "json"
    }

    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult> {
        let optimized = crate::postprocess::optimize_for_machine(model, opts);
        let bytes = crate::mapping::encode_model(&optimized).map_err(|e| {
            AppError::new(
                ReasonCode::IoNormalizeRounded,
                format!("Failed to encode JSON model: {e}"),
            )
        })?;
        let mut report = IoReport::new(self.format_id());
        report.entities_in = model.entities.len();
        report.entities_out = optimized.entities.len();
        report
            .extras
            .insert("postprocess".to_string(), opts.postprocess.to_string());
        Ok(ExportResult {
            bytes,
            warnings: Vec::new(),
            report,
        })
    }
}

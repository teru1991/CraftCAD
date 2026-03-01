use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::pipeline::finalize_export;
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
                ReasonCode::new("IO_LIMIT_016"),
                Severity::Error,
                format!("Failed to encode JSON model: {e}"),
            )
        })?;
        let report = IoReport {
            format: self.format_id().to_string(),
            entities_in: model.entities.len(),
            entities_out: optimized.entities.len(),
            postprocess_applied: opts.postprocess,
            ..IoReport::default()
        };
        Ok(finalize_export(bytes, report, Vec::new(), opts))
    }
}

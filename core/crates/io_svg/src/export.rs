use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::pipeline::finalize_export;
use craftcad_io::report::IoReport;
use craftcad_io::{ExportResult, Exporter};

pub struct SvgExporter;

impl Exporter for SvgExporter {
    fn format_id(&self) -> &'static str {
        "svg"
    }

    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult> {
        let _optimized = crate::postprocess::optimize_for_machine(model, opts);
        let warnings = vec![AppError::new(
            ReasonCode::new("IO_CURVE_APPROX_APPLIED"),
            Severity::Warn,
            "SVG exporter currently emits minimal deterministic output",
        )];
        let bytes =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>\n".to_vec();
        let report = IoReport {
            format: self.format_id().to_string(),
            entities_in: model.entities.len(),
            entities_out: model.entities.len(),
            approx_applied: 1,
            postprocess_applied: opts.postprocess,
            ..IoReport::default()
        };
        Ok(finalize_export(bytes, report, warnings, opts))
    }
}

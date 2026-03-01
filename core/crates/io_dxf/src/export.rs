use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::pipeline::finalize_export;
use craftcad_io::report::IoReport;
use craftcad_io::{ExportResult, Exporter};

pub struct DxfExporter;

impl Exporter for DxfExporter {
    fn format_id(&self) -> &'static str {
        "dxf"
    }

    fn export_bytes(&self, model: &InternalModel, opts: &ExportOptions) -> AppResult<ExportResult> {
        let (_optimized, approx_applied, origin_shifted) =
            crate::postprocess::optimize_for_machine(model, opts);
        let warnings = vec![AppError::new(
            ReasonCode::new("IO_APPROX_022"),
            Severity::Warn,
            "DXF exporter currently emits placeholder output",
        )];
        let placeholder = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n".to_vec();
        let report = IoReport {
            format: self.format_id().to_string(),
            entities_in: model.entities.len(),
            entities_out: model.entities.len(),
            approx_applied,
            postprocess_applied: opts.postprocess,
            origin_shifted,
            ..IoReport::default()
        };
        Ok(finalize_export(placeholder, report, warnings, opts))
    }
}

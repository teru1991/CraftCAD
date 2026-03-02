use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
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
            ReasonCode::IoNormalizeRounded,
            "DXF exporter currently emits placeholder output",
        )];
        let placeholder = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n".to_vec();
        let mut report = IoReport::new(self.format_id());
        report.entities_in = model.entities.len();
        report.entities_out = model.entities.len();
        report.approx_applied_count = approx_applied;
        report.origin_shifted = origin_shifted;
        report
            .extras
            .insert("postprocess".to_string(), opts.postprocess.to_string());
        Ok(ExportResult {
            bytes: placeholder,
            warnings,
            report,
        })
    }
}

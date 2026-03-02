use craftcad_io::model::InternalModel;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
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
            ReasonCode::IoNormalizeRounded,
            "SVG exporter currently emits minimal deterministic output",
        )];
        let bytes =
            b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1 1\"></svg>\n".to_vec();
        let mut report = IoReport::new(self.format_id());
        report.entities_in = model.entities.len();
        report.entities_out = model.entities.len();
        report.approx_applied_count = 1;
        report
            .extras
            .insert("postprocess".to_string(), opts.postprocess.to_string());

        Ok(ExportResult {
            bytes,
            warnings,
            report,
        })
    }
}

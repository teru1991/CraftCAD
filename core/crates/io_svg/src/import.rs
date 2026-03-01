use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use craftcad_io::options::ImportOptions;
use craftcad_io::pipeline::finalize_import;
use craftcad_io::report::IoReport;
use craftcad_io::{ImportResult, Importer};

pub struct SvgImporter;

impl Importer for SvgImporter {
    fn format_id(&self) -> &'static str {
        "svg"
    }

    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult> {
        crate::preflight::run(bytes, opts)?;
        let warnings = vec![AppError::new(
            ReasonCode::new("IO_CURVE_APPROX_APPLIED"),
            Severity::Warn,
            "SVG importer currently keeps best-effort placeholder behavior",
        )];
        let model = crate::mapping::empty_model(opts.seed, opts.approx_epsilon);
        let report = IoReport {
            format: self.format_id().to_string(),
            entities_in: 0,
            entities_out: 0,
            approx_applied: 1,
            unit_guessed: opts.allow_unit_guess,
            ..IoReport::default()
        };
        Ok(finalize_import(model, opts, report, warnings))
    }
}

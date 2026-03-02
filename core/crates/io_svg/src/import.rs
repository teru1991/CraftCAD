use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
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
            ReasonCode::IoNormalizeRounded,
            "SVG importer currently keeps best-effort placeholder behavior",
        )];
        let model = crate::mapping::empty_model(
            opts.determinism.seed,
            opts.determinism.round_step,
            opts.allow_unit_guess,
        );
        let mut report = IoReport::new(self.format_id());
        report.approx_applied_count = 1;
        report.unit_guessed = opts.allow_unit_guess;
        Ok(ImportResult {
            model,
            warnings,
            report,
        })
    }
}

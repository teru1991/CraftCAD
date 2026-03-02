use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use craftcad_io::{ImportResult, Importer};

pub struct DxfImporter;

impl Importer for DxfImporter {
    fn format_id(&self) -> &'static str {
        "dxf"
    }

    fn import_bytes(&self, bytes: &[u8], opts: &ImportOptions) -> AppResult<ImportResult> {
        crate::preflight::run(bytes, opts)?;
        let warnings = vec![AppError::new(
            ReasonCode::IoNormalizeRounded,
            "DXF importer is currently best-effort and returns an empty model",
        )];
        let model = crate::mapping::empty_model(opts.determinism.seed, opts.determinism.round_step);
        let mut report = IoReport::new(self.format_id());
        report.approx_applied_count = 1;
        Ok(ImportResult {
            model,
            warnings,
            report,
        })
    }
}

use crate::model::InternalModel;
use crate::normalize::normalize_model;
use crate::options::{ExportOptions, ImportOptions};
use crate::report::IoReport;
use crate::{ExportResult, ImportResult};

pub fn finalize_import(
    mut model: InternalModel,
    opts: &ImportOptions,
    mut report: IoReport,
    warnings: Vec<craftcad_errors::AppError>,
) -> ImportResult {
    normalize_model(&mut model, opts.approx_epsilon);
    report.entities_out = model.entities.len();
    ImportResult {
        model,
        warnings,
        report,
    }
}

pub fn finalize_export(
    bytes: Vec<u8>,
    report: IoReport,
    warnings: Vec<craftcad_errors::AppError>,
    _opts: &ExportOptions,
) -> ExportResult {
    ExportResult {
        bytes,
        warnings,
        report,
    }
}

use crate::approx::apply_approx;
use crate::model::InternalModel;
use crate::normalize::normalize_model;
use crate::options::{ExportOptions, ImportOptions};
use crate::path_opt::postprocess_model;
use crate::reasons::AppError;

pub fn run_shared_import_pipeline(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
) {
    normalize_model(model, opts.determinism.round_step, warnings);

    if opts.enable_approx {
        let seg = opts
            .determinism
            .approx_min_segments
            .max(2)
            .min(opts.determinism.approx_max_segments.max(2));
        apply_approx(model, seg, warnings);
    }

    if opts.enable_postprocess {
        postprocess_model(model, opts.determinism.close_eps, warnings);
    }
}

pub fn run_shared_export_pipeline(
    model: &mut InternalModel,
    opts: &ExportOptions,
    warnings: &mut Vec<AppError>,
) {
    normalize_model(model, opts.determinism.round_step, warnings);

    if opts.enable_approx {
        let seg = opts
            .determinism
            .approx_min_segments
            .max(2)
            .min(opts.determinism.approx_max_segments.max(2));
        apply_approx(model, seg, warnings);
    }

    if opts.enable_postprocess && opts.postprocess {
        postprocess_model(model, opts.determinism.close_eps, warnings);
    }
}

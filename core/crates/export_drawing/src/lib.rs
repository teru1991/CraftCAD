use drawing_model::DrawingDoc;
use drawing_style::{
    apply_bw_mode, apply_line_weight_scale, build_sheet_ir, load_bundle, render_svg,
    DrawingSsotBundle, ProjectMeta, SsotError, SsotPaths, SvgError,
};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("ssot error: {0}")]
    Ssot(#[from] SsotError),
    #[error("svg error: {0}")]
    Svg(#[from] SvgError),
}

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub style_preset_id: String,
    pub sheet_template_id: String,
    pub print_preset_id: String,
    pub meta: ProjectMeta,
}

pub struct DrawingExporter;

impl DrawingExporter {
    pub fn export_svg(
        repo_root: &Path,
        _drawing: Option<&DrawingDoc>,
        req: &ExportRequest,
    ) -> Result<String, ExportError> {
        let paths = SsotPaths::from_repo_root(repo_root);
        let bundle: DrawingSsotBundle = load_bundle(
            &paths,
            &req.style_preset_id,
            &req.sheet_template_id,
            &req.print_preset_id,
        )?;

        let mut ir = build_sheet_ir(&bundle, &req.meta);

        if bundle.print.color_mode == "bw" {
            apply_bw_mode(&mut ir);
        }
        apply_line_weight_scale(
            &mut ir,
            bundle.print.line_weight_scale,
            bundle.style.line_weights.min_line_weight_mm,
        );

        let svg = render_svg(&ir, bundle.print.export.svg_precision)?;
        Ok(svg)
    }
}

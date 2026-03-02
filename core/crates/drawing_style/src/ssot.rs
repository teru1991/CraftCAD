use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SsotError {
    #[error("failed to read file: {0}")]
    ReadFailed(String),
    #[error("failed to parse json: {0}")]
    ParseFailed(String),
    #[error("invalid preset id *_vN: {0}")]
    InvalidId(String),
    #[error("preset not found: {0}")]
    NotFound(String),
}

fn ensure_id_v(s: &str) -> Result<(), SsotError> {
    let re = Regex::new(r"^[a-z][a-z0-9_]*_v[0-9]+$").unwrap();
    if re.is_match(s) {
        Ok(())
    } else {
        Err(SsotError::InvalidId(s.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSsot {
    pub schema_version: u32,
    pub styles: Vec<StylePreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: String,
    pub label: I18nLabel,
    pub units: UnitsPolicy,
    pub rounding: RoundingPolicy,
    pub fonts: FontsPolicy,
    pub line_weights: LineWeightsPolicy,
    pub dimension: DimensionPolicy,
    pub linetypes: LinetypesPolicy,
    pub colors: ColorsPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nLabel {
    #[serde(default)]
    pub ja: Option<String>,
    #[serde(default)]
    pub en: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitsPolicy {
    pub internal: String,
    pub display: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundingPolicy {
    pub length_step: f64,
    pub angle_deg_step: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontsPolicy {
    pub primary_family: String,
    pub fallback_families: Vec<String>,
    pub size_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineWeightsPolicy {
    pub thin_mm: f64,
    pub normal_mm: f64,
    pub bold_mm: f64,
    pub scale_with_print: bool,
    pub min_line_weight_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowPolicy {
    #[serde(rename = "type")]
    pub ty: String,
    pub size_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderPolicy {
    pub default_angle_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionPolicy {
    pub arrow: ArrowPolicy,
    pub ext_line_gap_mm: f64,
    pub ext_line_overhang_mm: f64,
    pub dim_line_offset_mm: f64,
    pub dim_line_step_mm: f64,
    pub text_gap_mm: f64,
    pub text_box_padding_mm: f64,
    pub leader: LeaderPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinetypeDef {
    pub pattern_mm: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinetypesPolicy {
    pub solid: LinetypeDef,
    pub dashed: LinetypeDef,
    pub center: LinetypeDef,
    pub hidden: LinetypeDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsPolicy {
    pub by_layer: bool,
    pub default_stroke_hex: String,
    pub default_fill_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetTemplatesSsot {
    pub schema_version: u32,
    pub templates: Vec<SheetTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetTemplate {
    pub id: String,
    pub label: I18nLabel,
    pub page: PageDef,
    pub viewports: ViewportsDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDef {
    pub size: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub margin_mm: MarginsMm,
    pub title_block: TitleBlockDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginsMm {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectMm {
    pub x_mm: f64,
    pub y_mm: f64,
    pub w_mm: f64,
    pub h_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleBlockField {
    pub key: String,
    pub label: I18nLabel,
    pub required: bool,
    #[serde(default)]
    pub max_chars: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleBlockDef {
    pub position: String,
    pub bbox_mm: RectMm,
    pub fields: Vec<TitleBlockField>,
    pub field_font_size_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportsDef {
    pub model_view_region: RectMm,
    #[serde(default)]
    pub notes_region: Option<RectMm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPresetsSsot {
    pub schema_version: u32,
    pub presets: Vec<PrintPreset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintPreset {
    pub id: String,
    pub label: I18nLabel,
    pub target_template_id: String,
    pub margins_mm: MarginsMm,
    pub scale_policy: ScalePolicy,
    pub min_readable_text_height_mm: f64,
    pub line_weight_scale: f64,
    pub color_mode: String,
    pub text_hinting: TextHinting,
    pub export: ExportPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalePolicy {
    pub mode: String,
    #[serde(default)]
    pub fixed_scale: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextHinting {
    pub prefer_outlines: bool,
    pub kerning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPolicy {
    pub svg_precision: u32,
    pub pdf_embed_fonts: bool,
    pub pdf_page_rotation_deg: u32,
}

#[derive(Debug, Clone)]
pub struct DrawingSsotBundle {
    pub style: StylePreset,
    pub sheet: SheetTemplate,
    pub print: PrintPreset,
}

#[derive(Debug, Clone)]
pub struct SsotPaths {
    pub style_ssot_json: PathBuf,
    pub sheet_templates_json: PathBuf,
    pub print_presets_json: PathBuf,
}

impl SsotPaths {
    pub fn from_repo_root(repo_root: &Path) -> Self {
        let base = repo_root.join("docs/specs/drawing_style");
        Self {
            style_ssot_json: base.join("style_ssot.json"),
            sheet_templates_json: base.join("sheet_templates.json"),
            print_presets_json: base.join("print_presets.json"),
        }
    }
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, SsotError> {
    let s = fs::read_to_string(path)
        .map_err(|e| SsotError::ReadFailed(format!("{}: {}", path.display(), e)))?;
    serde_json::from_str(&s)
        .map_err(|e| SsotError::ParseFailed(format!("{}: {}", path.display(), e)))
}

pub fn load_bundle(
    paths: &SsotPaths,
    style_id: &str,
    sheet_id: &str,
    print_id: &str,
) -> Result<DrawingSsotBundle, SsotError> {
    ensure_id_v(style_id)?;
    ensure_id_v(sheet_id)?;
    ensure_id_v(print_id)?;

    let style_ssot: StyleSsot = read_json_file(&paths.style_ssot_json)?;
    let sheet_ssot: SheetTemplatesSsot = read_json_file(&paths.sheet_templates_json)?;
    let print_ssot: PrintPresetsSsot = read_json_file(&paths.print_presets_json)?;

    let style = style_ssot
        .styles
        .into_iter()
        .find(|s| s.id == style_id)
        .ok_or_else(|| SsotError::NotFound(style_id.to_string()))?;
    let sheet = sheet_ssot
        .templates
        .into_iter()
        .find(|t| t.id == sheet_id)
        .ok_or_else(|| SsotError::NotFound(sheet_id.to_string()))?;
    let print = print_ssot
        .presets
        .into_iter()
        .find(|p| p.id == print_id)
        .ok_or_else(|| SsotError::NotFound(print_id.to_string()))?;

    Ok(DrawingSsotBundle {
        style,
        sheet,
        print,
    })
}

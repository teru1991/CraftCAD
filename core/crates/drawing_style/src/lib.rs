use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const STYLE_SSOT_SCHEMA: &str =
    include_str!("../../../../docs/specs/drawing/style_ssot.schema.json");
const STYLE_SSOT: &str = include_str!("../../../../docs/specs/drawing/style_ssot.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreset {
    pub version: String,
    pub line_styles: Vec<LineStyle>,
    pub line_weights: BTreeMap<String, f64>,
    pub color_policy: ColorPolicy,
    pub fonts: FontSpec,
    pub text: TextSpec,
    pub arrow: ArrowSpec,
    pub rounding: RoundingPolicy,
    pub units_display: UnitsDisplay,
    pub dimension_rules: DimensionRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineStyle {
    pub name: String,
    pub pattern: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPolicy {
    pub mode: String,
    pub rgb: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSpec {
    pub family: String,
    pub fallback: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSpec {
    pub size_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowSpec {
    pub style: String,
    pub size_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundingPolicy {
    pub metric_decimals: u32,
    pub imperial_decimals: u32,
    pub mode: String,
    pub epsilon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitsDisplay {
    pub metric_suffix: String,
    pub imperial_suffix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionRules {
    pub extension_mm: f64,
    pub offset_mm: f64,
    pub text_gap_mm: f64,
    pub collision_step_mm: f64,
}

impl StylePreset {
    pub fn load_default() -> AppResult<Self> {
        let schema: serde_json::Value = serde_json::from_str(STYLE_SSOT_SCHEMA).map_err(|e| {
            AppError::new(
                ReasonCode::new("CAD_DRAWING_STYLE_SCHEMA_INVALID"),
                Severity::Fatal,
                format!("style_ssot.schema.json parse failed: {e}"),
            )
        })?;
        let data: serde_json::Value = serde_json::from_str(STYLE_SSOT).map_err(|e| {
            AppError::new(
                ReasonCode::new("CAD_DRAWING_STYLE_INVALID"),
                Severity::Error,
                format!("style_ssot.json parse failed: {e}"),
            )
        })?;

        let compiled = jsonschema::JSONSchema::compile(&schema).map_err(|e| {
            AppError::new(
                ReasonCode::new("CAD_DRAWING_STYLE_SCHEMA_INVALID"),
                Severity::Fatal,
                format!("style ssot schema compile failed: {e}"),
            )
        })?;

        if let Err(errors) = compiled.validate(&data) {
            let detail = errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
            return Err(AppError::new(
                ReasonCode::new("CAD_DRAWING_STYLE_INVALID"),
                Severity::Error,
                "style ssot validation failed",
            )
            .with_context("validation", detail));
        }

        serde_json::from_value(data).map_err(|e| {
            AppError::new(
                ReasonCode::new("CAD_DRAWING_STYLE_INVALID"),
                Severity::Error,
                format!("style ssot decode failed: {e}"),
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DimensionSpec {
    Linear {
        id: String,
        start: [f64; 2],
        end: [f64; 2],
        baseline: bool,
    },
    Angular {
        id: String,
        center: [f64; 2],
        p1: [f64; 2],
        p2: [f64; 2],
    },
    Radius {
        id: String,
        center: [f64; 2],
        edge: [f64; 2],
    },
    Diameter {
        id: String,
        p1: [f64; 2],
        p2: [f64; 2],
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationSpec {
    Text {
        id: String,
        at: [f64; 2],
        value: String,
    },
    Leader {
        id: String,
        anchor: [f64; 2],
        text_at: [f64; 2],
        value: String,
    },
    HoleCallout {
        id: String,
        at: [f64; 2],
        diameter: f64,
    },
    ChamferCallout {
        id: String,
        at: [f64; 2],
        size: f64,
        angle_deg: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderPlan {
    pub commands: Vec<RenderCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderCommand {
    DimensionLine {
        key: String,
        from: [f64; 2],
        to: [f64; 2],
    },
    DimensionText {
        key: String,
        at: [f64; 2],
        value: String,
    },
    AnnotationText {
        key: String,
        at: [f64; 2],
        value: String,
    },
    LeaderLine {
        key: String,
        from: [f64; 2],
        to: [f64; 2],
    },
}

pub fn build_render_plan(
    style: &StylePreset,
    dimensions: &[DimensionSpec],
    annotations: &[AnnotationSpec],
) -> AppResult<RenderPlan> {
    let mut commands = Vec::new();

    let mut dim_sorted = dimensions.to_vec();
    dim_sorted.sort_by(|a, b| dim_id(a).cmp(dim_id(b)));

    for (idx, dim) in dim_sorted.iter().enumerate() {
        match dim {
            DimensionSpec::Linear { id, start, end, .. } => {
                if length(*start, *end) <= style.rounding.epsilon {
                    return Err(AppError::new(
                        ReasonCode::new("CAD_DIMENSION_ZERO_LENGTH"),
                        Severity::Error,
                        format!("linear dimension {id} has zero length"),
                    ));
                }
                let lane = idx as f64 * style.dimension_rules.collision_step_mm;
                commands.push(RenderCommand::DimensionLine {
                    key: id.clone(),
                    from: round_point(shift(*start, lane), style.rounding.metric_decimals),
                    to: round_point(shift(*end, lane), style.rounding.metric_decimals),
                });
            }
            DimensionSpec::Angular { id, center, p1, p2 } => {
                if length(*center, *p1) <= style.rounding.epsilon
                    || length(*center, *p2) <= style.rounding.epsilon
                {
                    return Err(AppError::new(
                        ReasonCode::new("CAD_DIMENSION_GEOMETRY_INSUFFICIENT"),
                        Severity::Error,
                        format!("angular dimension {id} has invalid ray"),
                    ));
                }
                let label = format!(
                    "{}°",
                    round(length(*p1, *p2), style.rounding.metric_decimals)
                );
                commands.push(RenderCommand::DimensionText {
                    key: id.clone(),
                    at: round_point(*center, style.rounding.metric_decimals),
                    value: label,
                });
            }
            DimensionSpec::Radius { id, center, edge } => {
                let r = length(*center, *edge);
                if r <= style.rounding.epsilon {
                    return Err(AppError::new(
                        ReasonCode::new("CAD_DIMENSION_GEOMETRY_INSUFFICIENT"),
                        Severity::Error,
                        format!("radius dimension {id} is degenerate"),
                    ));
                }
                commands.push(RenderCommand::DimensionText {
                    key: id.clone(),
                    at: round_point(*edge, style.rounding.metric_decimals),
                    value: format!("R{}", round(r, style.rounding.metric_decimals)),
                });
            }
            DimensionSpec::Diameter { id, p1, p2 } => {
                let d = length(*p1, *p2);
                if d <= style.rounding.epsilon {
                    return Err(AppError::new(
                        ReasonCode::new("CAD_DIMENSION_ZERO_LENGTH"),
                        Severity::Error,
                        format!("diameter dimension {id} is degenerate"),
                    ));
                }
                commands.push(RenderCommand::DimensionText {
                    key: id.clone(),
                    at: round_point(mid(*p1, *p2), style.rounding.metric_decimals),
                    value: format!("⌀{}", round(d, style.rounding.metric_decimals)),
                });
            }
        }
    }

    let mut ann_sorted = annotations.to_vec();
    ann_sorted.sort_by(|a, b| ann_id(a).cmp(ann_id(b)));

    for ann in ann_sorted {
        match ann {
            AnnotationSpec::Text { id, at, value } => {
                validate_annotation_text(&value)?;
                commands.push(RenderCommand::AnnotationText {
                    key: id,
                    at: round_point(at, style.rounding.metric_decimals),
                    value,
                });
            }
            AnnotationSpec::Leader {
                id,
                anchor,
                text_at,
                value,
            } => {
                validate_annotation_text(&value)?;
                commands.push(RenderCommand::LeaderLine {
                    key: id.clone(),
                    from: round_point(anchor, style.rounding.metric_decimals),
                    to: round_point(text_at, style.rounding.metric_decimals),
                });
                commands.push(RenderCommand::AnnotationText {
                    key: id,
                    at: round_point(text_at, style.rounding.metric_decimals),
                    value,
                });
            }
            AnnotationSpec::HoleCallout { id, at, diameter } => {
                commands.push(RenderCommand::AnnotationText {
                    key: id,
                    at: round_point(at, style.rounding.metric_decimals),
                    value: format!("⌀{}", round(diameter, style.rounding.metric_decimals)),
                });
            }
            AnnotationSpec::ChamferCallout {
                id,
                at,
                size,
                angle_deg,
            } => {
                commands.push(RenderCommand::AnnotationText {
                    key: id,
                    at: round_point(at, style.rounding.metric_decimals),
                    value: format!("C{}x{}°", round(size, 2), round(angle_deg, 1)),
                });
            }
        }
    }

    Ok(RenderPlan { commands })
}

fn validate_annotation_text(value: &str) -> AppResult<()> {
    if value.chars().count() > 120 {
        return Err(AppError::new(
            ReasonCode::new("CAD_ANNOT_TEXT_TOO_LONG"),
            Severity::Error,
            "annotation text exceeds 120 chars",
        ));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
    {
        return Err(AppError::new(
            ReasonCode::new("CAD_ANNOT_UNSUPPORTED_CHAR"),
            Severity::Error,
            "annotation text contains unsupported control character",
        ));
    }
    Ok(())
}

fn dim_id(d: &DimensionSpec) -> &str {
    match d {
        DimensionSpec::Linear { id, .. }
        | DimensionSpec::Angular { id, .. }
        | DimensionSpec::Radius { id, .. }
        | DimensionSpec::Diameter { id, .. } => id,
    }
}

fn ann_id(a: &AnnotationSpec) -> &str {
    match a {
        AnnotationSpec::Text { id, .. }
        | AnnotationSpec::Leader { id, .. }
        | AnnotationSpec::HoleCallout { id, .. }
        | AnnotationSpec::ChamferCallout { id, .. } => id,
    }
}

fn round(v: f64, decimals: u32) -> f64 {
    let p = 10_f64.powi(decimals as i32);
    (v * p).round() / p
}

fn round_point(p: [f64; 2], decimals: u32) -> [f64; 2] {
    [round(p[0], decimals), round(p[1], decimals)]
}

fn shift(p: [f64; 2], y: f64) -> [f64; 2] {
    [p[0], p[1] + y]
}

fn length(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    (dx * dx + dy * dy).sqrt()
}

fn mid(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

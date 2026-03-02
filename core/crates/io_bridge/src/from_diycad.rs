use craftcad_diycad::{load, DiycadProject};
use craftcad_io::model::*;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct LoadDiycadOptions {
    pub path: String,
}

impl LoadDiycadOptions {
    pub fn default_for_tests(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BridgeEntity {
    Path {
        id: String,
        layer: String,
        linetype: String,
        closed: bool,
        points: Vec<(f64, f64)>,
    },
    Text {
        id: String,
        layer: String,
        x: f64,
        y: f64,
        text: String,
        size: f32,
    },
}

pub fn load_diycad_to_internal_model(
    opts: &LoadDiycadOptions,
) -> AppResult<(InternalModel, Vec<AppError>)> {
    let mut warnings = Vec::new();

    let project: DiycadProject = load(&opts.path).map_err(|e| {
        AppError::new(
            ReasonCode::LOAD_DIYCAD_READ_FAILED,
            "failed to read .diycad",
        )
        .with_context("path", opts.path.clone())
        .with_context("error", e.to_string())
        .fatal()
    })?;

    let units = match project.manifest.units.as_str() {
        "inch" => Units::Inch,
        _ => Units::Mm,
    };

    let mut model = InternalModel::new(units);
    model.metadata.source_format = "diycad".to_string();

    for raw in &project.data.entities {
        let parsed: BridgeEntity = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                warnings.push(
                    AppError::new(
                        ReasonCode::LOAD_PART_EXPORT_FAILED,
                        "failed to parse bridge entity; dropped",
                    )
                    .with_context("error", e.to_string()),
                );
                continue;
            }
        };

        match parsed {
            BridgeEntity::Path {
                id,
                layer,
                linetype,
                closed,
                points,
            } => {
                let mut p = PathEntity::new(
                    id,
                    StrokeStyle {
                        layer,
                        linetype,
                        weight: 0.25,
                        color_policy: ColorPolicy::ByLayer,
                    },
                );
                p.closed = closed;
                for w in points.windows(2) {
                    p.segments.push(Segment2D::Line {
                        a: Point2D {
                            x: w[0].0,
                            y: w[0].1,
                        },
                        b: Point2D {
                            x: w[1].0,
                            y: w[1].1,
                        },
                    });
                }
                model.entities.push(Entity::Path(p));
            }
            BridgeEntity::Text {
                id,
                layer,
                x,
                y,
                text,
                size,
            } => {
                model.entities.push(Entity::Text(TextEntity {
                    id,
                    layer,
                    pos: Point2D { x, y },
                    text,
                    size,
                    font_hint: None,
                    rotation_rad: 0.0,
                }));
            }
        }
    }

    if model.entities.is_empty() {
        warnings.push(
            AppError::new(
                ReasonCode::LOAD_DIYCAD_EMPTY,
                "loaded .diycad but no entities found",
            )
            .with_context("path", opts.path.clone()),
        );
    }

    Ok((model, warnings))
}

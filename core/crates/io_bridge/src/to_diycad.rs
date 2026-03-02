use craftcad_diycad::{create_empty_project, save, DataJson};
use craftcad_io::model::*;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SaveDiycadOptions {
    pub path: String,
    pub project_name: String,
    pub part_name: String,
    pub allow_drop: bool,
}

impl SaveDiycadOptions {
    pub fn default_for_tests(path: &str) -> Self {
        Self {
            path: path.to_string(),
            project_name: "test".to_string(),
            part_name: "part1".to_string(),
            allow_drop: true,
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

pub fn save_internal_model_to_diycad(
    model: &InternalModel,
    opts: &SaveDiycadOptions,
) -> AppResult<Vec<AppError>> {
    let mut warnings = Vec::new();

    let timestamp = "2026-01-01T00:00:00Z";
    let mut project = create_empty_project("io_bridge", model.units.as_str(), timestamp);
    let mut entities: Vec<String> = Vec::new();

    for e in &model.entities {
        match e {
            Entity::Path(p) => {
                let mut points: Vec<(f64, f64)> = Vec::new();
                let mut ok = true;

                for s in &p.segments {
                    match s {
                        Segment2D::Line { a, b } => {
                            if points.is_empty() {
                                points.push((a.x, a.y));
                            }
                            points.push((b.x, b.y));
                        }
                        other => {
                            ok = false;
                            if opts.allow_drop {
                                warnings.push(
                                    AppError::new(
                                        ReasonCode::SAVE_GEOM_DROPPED,
                                        "non-line segment dropped for diycad save",
                                    )
                                    .with_context("path_id", p.id.clone())
                                    .with_context("segment_kind", format!("{:?}", other))
                                    .with_hint("enable_approx=true で事前Line化すると保存品質が向上します。"),
                                );
                            } else {
                                return Err(AppError::new(
                                    ReasonCode::SAVE_PART_IMPORT_FAILED,
                                    "cannot save non-line segments without allow_drop",
                                )
                                .with_context("path_id", p.id.clone())
                                .fatal());
                            }
                        }
                    }
                }

                if ok && points.len() >= 2 {
                    let be = BridgeEntity::Path {
                        id: p.id.clone(),
                        layer: p.stroke.layer.clone(),
                        linetype: p.stroke.linetype.clone(),
                        closed: p.closed,
                        points,
                    };
                    entities.push(serde_json::to_string(&be).map_err(|e| {
                        AppError::new(
                            ReasonCode::SAVE_PART_IMPORT_FAILED,
                            "failed to serialize path bridge entity",
                        )
                        .with_context("error", e.to_string())
                        .fatal()
                    })?);
                }
            }
            Entity::Text(t) => {
                warnings.push(
                    AppError::new(
                        ReasonCode::SAVE_TEXT_BEST_EFFORT,
                        "text saved best-effort (font hint not embedded)",
                    )
                    .with_context("text_id", t.id.clone()),
                );
                let be = BridgeEntity::Text {
                    id: t.id.clone(),
                    layer: t.layer.clone(),
                    x: t.pos.x,
                    y: t.pos.y,
                    text: t.text.clone(),
                    size: t.size,
                };
                entities.push(serde_json::to_string(&be).map_err(|e| {
                    AppError::new(
                        ReasonCode::SAVE_PART_IMPORT_FAILED,
                        "failed to serialize text bridge entity",
                    )
                    .with_context("error", e.to_string())
                    .fatal()
                })?);
            }
        }
    }

    project.data = DataJson { entities };
    project.manifest.app_version = opts.project_name.clone();

    save(&opts.path, &project).map_err(|e| {
        AppError::new(
            ReasonCode::SAVE_DIYCAD_WRITE_FAILED,
            "failed to write .diycad",
        )
        .with_context("path", opts.path.clone())
        .with_context("error", e.to_string())
        .fatal()
    })?;

    Ok(warnings)
}

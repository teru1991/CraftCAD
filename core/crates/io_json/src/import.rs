use crate::schema::validate_v1;
use craftcad_io::model::*;
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use serde_json::Value;

fn parse_json(bytes: &[u8]) -> AppResult<Value> {
    serde_json::from_slice(bytes).map_err(|e| {
        AppError::new(ReasonCode::IO_PARSE_JSON_MALFORMED, "malformed json")
            .with_context("error", e.to_string())
            .fatal()
    })
}

fn units_from_str(s: &str) -> Units {
    match s {
        "inch" => Units::Inch,
        _ => Units::Mm,
    }
}

fn parse_color_policy(v: &Value) -> AppResult<ColorPolicy> {
    if let Some(s) = v.as_str() {
        if s == "by_layer" {
            return Ok(ColorPolicy::ByLayer);
        }
    }
    if let Some(obj) = v.as_object() {
        if let Some(rgb) = obj.get("fixed_rgb").and_then(|x| x.as_object()) {
            let r = rgb.get("r").and_then(|x| x.as_i64()).unwrap_or(0) as u8;
            let g = rgb.get("g").and_then(|x| x.as_i64()).unwrap_or(0) as u8;
            let b = rgb.get("b").and_then(|x| x.as_i64()).unwrap_or(0) as u8;
            return Ok(ColorPolicy::FixedRgb { r, g, b });
        }
    }
    Err(AppError::new(
        ReasonCode::IO_JSON_SCHEMA_INVALID,
        "invalid color_policy",
    ))
}

fn parse_point(o: &serde_json::Map<String, Value>) -> AppResult<Point2D> {
    let x = o.get("x").and_then(|v| v.as_f64()).ok_or_else(|| {
        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "point.x must be number")
    })?;
    let y = o.get("y").and_then(|v| v.as_f64()).ok_or_else(|| {
        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "point.y must be number")
    })?;
    Ok(Point2D { x, y })
}

fn parse_segment(v: &Value) -> AppResult<Segment2D> {
    let obj = v.as_object().ok_or_else(|| {
        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "segment must be object")
    })?;
    let kind = obj.get("kind").and_then(|v| v.as_str()).ok_or_else(|| {
        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "segment.kind required")
    })?;
    match kind {
        "line" => {
            let a = parse_point(obj.get("a").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "line.a required")
            })?)?;
            let b = parse_point(obj.get("b").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "line.b required")
            })?)?;
            Ok(Segment2D::Line { a, b })
        }
        "arc" => {
            let center = parse_point(obj.get("center").and_then(|v| v.as_object()).ok_or_else(
                || AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.center required"),
            )?)?;
            let radius = obj.get("radius").and_then(|v| v.as_f64()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.radius required")
            })?;
            let start_rad = obj
                .get("start_rad")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| {
                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.start_rad required")
                })?;
            let end_rad = obj.get("end_rad").and_then(|v| v.as_f64()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.end_rad required")
            })?;
            let ccw = obj.get("ccw").and_then(|v| v.as_bool()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.ccw required")
            })?;
            Ok(Segment2D::Arc {
                center,
                radius,
                start_rad,
                end_rad,
                ccw,
            })
        }
        "circle" => {
            let center = parse_point(obj.get("center").and_then(|v| v.as_object()).ok_or_else(
                || AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "circle.center required"),
            )?)?;
            let radius = obj.get("radius").and_then(|v| v.as_f64()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "circle.radius required")
            })?;
            Ok(Segment2D::Circle { center, radius })
        }
        "cubic_bezier" => {
            let a = parse_point(obj.get("a").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.a required")
            })?)?;
            let c1 = parse_point(obj.get("c1").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.c1 required")
            })?)?;
            let c2 = parse_point(obj.get("c2").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.c2 required")
            })?)?;
            let b = parse_point(obj.get("b").and_then(|v| v.as_object()).ok_or_else(|| {
                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.b required")
            })?)?;
            Ok(Segment2D::CubicBezier { a, c1, c2, b })
        }
        _ => Err(
            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown segment kind")
                .with_context("kind", kind),
        ),
    }
}

fn parse_entity(v: &Value) -> AppResult<Entity> {
    let obj = v.as_object().ok_or_else(|| {
        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "entity must be object")
    })?;
    let ty = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "entity.type required"))?;
    match ty {
        "path" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("path")
                .to_string();
            let closed = obj.get("closed").and_then(|v| v.as_bool()).unwrap_or(false);
            let stroke = obj
                .get("stroke")
                .and_then(|v| v.as_object())
                .ok_or_else(|| {
                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "path.stroke required")
                })?;
            let layer = stroke
                .get("layer")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let linetype = stroke
                .get("linetype")
                .and_then(|v| v.as_str())
                .unwrap_or("CONTINUOUS")
                .to_string();
            let weight = stroke
                .get("weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.25) as f32;
            let color_policy =
                parse_color_policy(stroke.get("color_policy").ok_or_else(|| {
                    AppError::new(
                        ReasonCode::IO_JSON_SCHEMA_INVALID,
                        "stroke.color_policy required",
                    )
                })?)?;

            let segs = obj
                .get("segments")
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    AppError::new(
                        ReasonCode::IO_JSON_SCHEMA_INVALID,
                        "path.segments must be array",
                    )
                })?;
            let mut segments = Vec::with_capacity(segs.len());
            for s in segs {
                segments.push(parse_segment(s)?);
            }

            let mut p = PathEntity::new(
                id,
                StrokeStyle {
                    layer,
                    linetype,
                    weight,
                    color_policy,
                },
            );
            p.closed = closed;
            p.segments = segments;
            Ok(Entity::Path(p))
        }
        "text" => {
            let id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("text")
                .to_string();
            let layer = obj
                .get("layer")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let pos =
                parse_point(obj.get("pos").and_then(|v| v.as_object()).ok_or_else(|| {
                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "text.pos required")
                })?)?;
            let text = obj
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let size = obj.get("size").and_then(|v| v.as_f64()).unwrap_or(10.0) as f32;
            let font_hint = obj
                .get("font_hint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let rotation_rad = obj
                .get("rotation_rad")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            Ok(Entity::Text(TextEntity {
                id,
                layer,
                pos,
                text,
                size,
                font_hint,
                rotation_rad,
            }))
        }
        _ => Err(
            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown entity.type")
                .with_context("type", ty),
        ),
    }
}

fn migrate_if_needed(
    doc: Value,
    _opts: &ImportOptions,
    _warnings: &mut [AppError],
    _report: &mut IoReport,
) -> AppResult<Value> {
    Ok(doc)
}

pub fn import_json(
    bytes: &[u8],
    opts: &ImportOptions,
) -> AppResult<(InternalModel, Vec<AppError>, IoReport)> {
    let mut warnings: Vec<AppError> = Vec::new();
    let mut report = IoReport::new("json");

    let doc = parse_json(bytes)?;
    validate_v1(&doc, &mut report)?;
    let doc = migrate_if_needed(doc, opts, &mut warnings, &mut report)?;

    let obj = doc.as_object().expect("validated json object");
    let units = units_from_str(obj.get("units").and_then(|v| v.as_str()).unwrap_or("mm"));
    let mut model = InternalModel::new(units);

    if let Some(meta) = obj.get("metadata").and_then(|v| v.as_object()) {
        model.metadata.source_format = meta
            .get("source_format")
            .and_then(|v| v.as_str())
            .unwrap_or("json")
            .to_string();
        model.metadata.unit_guess = meta
            .get("unit_guess")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        model.metadata.determinism_tag = meta
            .get("determinism_tag")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
    }

    if let Some(ents) = obj.get("entities").and_then(|v| v.as_array()) {
        for e in ents {
            model.entities.push(parse_entity(e)?);
        }
    }

    report.entities_in = model.entities.len();
    report.texts_in = model.texts.len();

    Ok((model, warnings, report))
}

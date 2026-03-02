use crate::mapping::map_stroke;
use crate::parse::{parse_dxf_groups, split_entities};
use craftcad_io::model::*;
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use craftcad_io_support::{MappingRules, SupportLevel, SupportMatrix};

fn get_f64(groups: &[(i32, String)], code: i32) -> Option<f64> {
    groups
        .iter()
        .find(|(c, _)| *c == code)
        .and_then(|(_, v)| v.trim().parse::<f64>().ok())
}
fn get_str(groups: &[(i32, String)], code: i32) -> Option<String> {
    groups
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, v)| v.clone())
}

pub fn import_dxf(
    bytes: &[u8],
    opts: &ImportOptions,
) -> AppResult<(InternalModel, Vec<AppError>, IoReport)> {
    let mut warnings = Vec::new();
    let mut report = IoReport::new("dxf");
    let sm = SupportMatrix::load_from_ssot()?;
    let mr = MappingRules::load_from_ssot()?;

    let groups = parse_dxf_groups(bytes, opts)?;
    let ents = split_entities(&groups);

    let mut model = InternalModel::new(crate::mapping::map_units(&mr, Units::Mm));
    model.metadata.source_format = "dxf".to_string();
    model.metadata.determinism_tag = opts.determinism_tag();

    for e in ents {
        let kind = e.kind.to_uppercase();
        let stroke = map_stroke(
            &mr,
            StrokeStyle {
                layer: e.layer.clone(),
                linetype: e.linetype.clone(),
                ..Default::default()
            },
        );
        let g2: Vec<(i32, String)> = e.groups.iter().map(|g| (g.code, g.value.clone())).collect();

        match kind.as_str() {
            "LINE" => {
                if sm.level("dxf", "entity_line", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let x1 = get_f64(&g2, 10).unwrap_or(0.0);
                let y1 = get_f64(&g2, 20).unwrap_or(0.0);
                let x2 = get_f64(&g2, 11).unwrap_or(0.0);
                let y2 = get_f64(&g2, 21).unwrap_or(0.0);
                let mut p = PathEntity::new(format!("dxf_line_{}", model.entities.len()), stroke);
                p.segments.push(Segment2D::Line {
                    a: Point2D { x: x1, y: y1 },
                    b: Point2D { x: x2, y: y2 },
                });
                model.entities.push(Entity::Path(p));
            }
            "LWPOLYLINE" | "POLYLINE" => {
                if sm.level("dxf", "entity_polyline", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let mut pts = Vec::new();
                let mut i = 0usize;
                while i < g2.len() {
                    if g2[i].0 == 10 {
                        let x = g2[i].1.trim().parse::<f64>().unwrap_or(0.0);
                        let y = g2
                            .iter()
                            .skip(i + 1)
                            .find(|(c, _)| *c == 20)
                            .and_then(|(_, v)| v.trim().parse::<f64>().ok())
                            .unwrap_or(0.0);
                        pts.push(Point2D { x, y });
                    }
                    i += 1;
                }
                if pts.len() < 2 {
                    continue;
                }
                let mut p = PathEntity::new(format!("dxf_pl_{}", model.entities.len()), stroke);
                for w in pts.windows(2) {
                    p.segments.push(Segment2D::Line { a: w[0], b: w[1] });
                }
                model.entities.push(Entity::Path(p));
            }
            "ARC" => {
                if sm.level("dxf", "entity_arc", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let cx = get_f64(&g2, 10).unwrap_or(0.0);
                let cy = get_f64(&g2, 20).unwrap_or(0.0);
                let r = get_f64(&g2, 40).unwrap_or(0.0);
                let a0 = get_f64(&g2, 50).unwrap_or(0.0).to_radians();
                let a1 = get_f64(&g2, 51).unwrap_or(0.0).to_radians();
                let mut p = PathEntity::new(format!("dxf_arc_{}", model.entities.len()), stroke);
                p.segments.push(Segment2D::Arc {
                    center: Point2D { x: cx, y: cy },
                    radius: r,
                    start_rad: a0,
                    end_rad: a1,
                    ccw: true,
                });
                model.entities.push(Entity::Path(p));
            }
            "CIRCLE" => {
                if sm.level("dxf", "entity_circle", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let cx = get_f64(&g2, 10).unwrap_or(0.0);
                let cy = get_f64(&g2, 20).unwrap_or(0.0);
                let r = get_f64(&g2, 40).unwrap_or(0.0);
                let mut p = PathEntity::new(format!("dxf_circle_{}", model.entities.len()), stroke);
                p.segments.push(Segment2D::Circle {
                    center: Point2D { x: cx, y: cy },
                    radius: r,
                });
                model.entities.push(Entity::Path(p));
            }
            "TEXT" | "MTEXT" => {
                let lvl = sm.level("dxf", "entity_text", "import");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }
                let t = TextEntity {
                    id: format!("dxf_text_{}", model.entities.len()),
                    layer: stroke.layer.clone(),
                    pos: Point2D {
                        x: get_f64(&g2, 10).unwrap_or(0.0),
                        y: get_f64(&g2, 20).unwrap_or(0.0),
                    },
                    text: get_str(&g2, 1).unwrap_or_default(),
                    size: get_f64(&g2, 40).unwrap_or(12.0) as f32,
                    font_hint: None,
                    rotation_rad: get_f64(&g2, 50).unwrap_or(0.0).to_radians(),
                };
                if lvl == SupportLevel::BestEffort {
                    for r in sm.reasons("dxf", "entity_text", "import") {
                        warnings.push(AppError::new(r, "DXF TEXT imported best-effort"));
                    }
                }
                model.entities.push(Entity::Text(t));
            }
            "SPLINE" => {
                let lvl = sm.level("dxf", "entity_spline", "import");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }
                let xs: Vec<f64> = g2
                    .iter()
                    .filter_map(|(c, v)| {
                        if *c == 10 {
                            v.parse::<f64>().ok()
                        } else {
                            None
                        }
                    })
                    .collect();
                let ys: Vec<f64> = g2
                    .iter()
                    .filter_map(|(c, v)| {
                        if *c == 20 {
                            v.parse::<f64>().ok()
                        } else {
                            None
                        }
                    })
                    .collect();
                let mut pts: Vec<Point2D> = Vec::new();
                let n = xs.len().min(ys.len());
                for i in 0..n {
                    pts.push(Point2D { x: xs[i], y: ys[i] });
                }
                let mut p = PathEntity::new(format!("dxf_spline_{}", model.entities.len()), stroke);
                if pts.len() >= 4 {
                    let mut i = 0usize;
                    while i + 3 < pts.len() {
                        p.segments.push(Segment2D::CubicBezier {
                            a: pts[i],
                            c1: pts[i + 1],
                            c2: pts[i + 2],
                            b: pts[i + 3],
                        });
                        i += 3;
                    }
                    warnings.push(
                        AppError::new(
                            ReasonCode::IO_DXF_SPLINE_CONVERTED,
                            "DXF SPLINE converted to cubic beziers",
                        )
                        .with_context("bezier_segments", p.segments.len().to_string()),
                    );
                } else if pts.len() >= 2 {
                    for w in pts.windows(2) {
                        p.segments.push(Segment2D::Line { a: w[0], b: w[1] });
                    }
                    warnings.push(
                        AppError::new(
                            ReasonCode::IO_DXF_SPLINE_CONVERTED,
                            "DXF SPLINE converted to polyline fallback",
                        )
                        .with_context("line_segments", p.segments.len().to_string()),
                    );
                }
                for r in sm.reasons("dxf", "entity_spline", "import") {
                    warnings.push(AppError::new(r, "DXF SPLINE best-effort conversion"));
                }
                if !p.segments.is_empty() {
                    model.entities.push(Entity::Path(p));
                }
            }
            other => warnings.push(
                AppError::new(
                    ReasonCode::IO_DXF_ENTITY_UNKNOWN_DROPPED,
                    "unknown DXF entity dropped",
                )
                .with_context("entity", other.to_string()),
            ),
        }
    }

    report.entities_in = model.entities.len();
    report.entities_out = model.entities.len();
    report.determinism_tag = opts.determinism_tag();
    Ok((model, warnings, report))
}

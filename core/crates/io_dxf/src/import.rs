use crate::mapping::map_stroke;
use crate::parse::{parse_dxf_groups, parse_header_insunits, split_entities};
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

fn get_i32(groups: &[(i32, String)], code: i32) -> Option<i32> {
    groups
        .iter()
        .find(|(c, _)| *c == code)
        .and_then(|(_, v)| v.trim().parse::<i32>().ok())
}

fn collect_text(groups: &[(i32, String)]) -> String {
    let mut out: Vec<String> = Vec::new();
    for (c, v) in groups {
        if (*c == 1 || *c == 3) && !v.is_empty() {
            out.push(v.clone());
        }
    }
    if out.is_empty() {
        String::new()
    } else {
        out.join("")
    }
}

#[derive(Debug, Clone, Copy)]
struct Vertex {
    p: Point2D,
    bulge: f64,
}

fn dist(a: Point2D, b: Point2D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn bulge_to_arc(a: Point2D, b: Point2D, bulge: f64) -> Option<Segment2D> {
    let chord = dist(a, b);
    if !chord.is_finite() || chord <= 0.0 || !bulge.is_finite() {
        return None;
    }
    let eps = 1e-12;
    if bulge.abs() <= eps {
        return Some(Segment2D::Line { a, b });
    }

    let theta = 4.0 * bulge.atan();
    let half = chord * 0.5;
    let sin_half = (theta * 0.5).sin();
    if sin_half.abs() <= eps {
        return Some(Segment2D::Line { a, b });
    }
    let r = half / sin_half.abs();

    let mut h2 = (r * r) - (half * half);
    if h2 < 0.0 {
        h2 = 0.0;
    }
    let h = h2.sqrt();

    let dx = (b.x - a.x) / chord;
    let dy = (b.y - a.y) / chord;
    let nx = -dy;
    let ny = dx;

    let mx = (a.x + b.x) * 0.5;
    let my = (a.y + b.y) * 0.5;

    let sign = if bulge >= 0.0 { 1.0 } else { -1.0 };
    let cx = mx + nx * h * sign;
    let cy = my + ny * h * sign;

    let center = Point2D { x: cx, y: cy };
    let start = (a.y - cy).atan2(a.x - cx);
    let end = (b.y - cy).atan2(b.x - cx);

    Some(Segment2D::Arc {
        center,
        radius: r,
        start_rad: start,
        end_rad: end,
        ccw: bulge >= 0.0,
    })
}

fn parse_polyline_vertices(g2: &[(i32, String)]) -> (Vec<Vertex>, bool) {
    let flags = get_i32(g2, 70).unwrap_or(0);
    let closed = (flags & 1) != 0;

    let mut verts: Vec<Vertex> = Vec::new();
    let mut pending_x: Option<f64> = None;
    let mut last_idx: Option<usize> = None;

    for (c, v) in g2 {
        match *c {
            10 => {
                pending_x = v.trim().parse::<f64>().ok();
            }
            20 => {
                if let Some(x) = pending_x.take() {
                    if let Ok(y) = v.trim().parse::<f64>() {
                        verts.push(Vertex {
                            p: Point2D { x, y },
                            bulge: 0.0,
                        });
                        last_idx = Some(verts.len() - 1);
                    }
                }
            }
            42 => {
                if let Some(i) = last_idx {
                    if let Ok(bulge) = v.trim().parse::<f64>() {
                        verts[i].bulge = bulge;
                    }
                }
            }
            _ => {}
        }
    }

    (verts, closed)
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
    let insunits = parse_header_insunits(&groups);
    let ents = split_entities(&groups);

    let mut units = mr.default_units();
    if let Some(u) = insunits {
        units = match u {
            1 => Units::Inch,
            4 => Units::Mm,
            other => {
                if opts.allow_unit_guess {
                    warnings.push(
                        AppError::new(
                            ReasonCode::IO_UNIT_GUESSED,
                            "unknown INSUNITS; falling back to mapping_rules default",
                        )
                        .with_context("INSUNITS", other.to_string())
                        .with_context("fallback", units.as_str()),
                    );
                }
                units
            }
        };
    } else if opts.allow_unit_guess {
        warnings.push(
            AppError::new(
                ReasonCode::IO_UNIT_GUESSED,
                "INSUNITS not found; using mapping_rules default",
            )
            .with_context("fallback", units.as_str()),
        );
    }

    let mut model = InternalModel::new(crate::mapping::map_units(&mr, units));
    model.metadata.source_format = "dxf".to_string();
    model.metadata.determinism_tag = opts.determinism_tag();
    if let Some(u) = insunits {
        model.metadata.unit_guess = Some(format!("header:$INSUNITS={}", u));
    }

    for e in ents {
        let kind = e.kind.to_uppercase();
        let stroke = map_stroke(
            &mr,
            StrokeStyle {
                layer: e.layer.clone(),
                linetype: e.linetype.clone(),
                ..StrokeStyle::default()
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
                let (verts, closed) = parse_polyline_vertices(&g2);
                if verts.len() < 2 {
                    continue;
                }

                let mut p = PathEntity::new(format!("dxf_pl_{}", model.entities.len()), stroke);
                p.closed = closed;

                for i in 0..(verts.len() - 1) {
                    let a = verts[i].p;
                    let b = verts[i + 1].p;
                    let bulge = verts[i].bulge;
                    if let Some(seg) = bulge_to_arc(a, b, bulge) {
                        p.segments.push(seg);
                    }
                }
                if closed {
                    let a = verts[verts.len() - 1].p;
                    let b = verts[0].p;
                    let bulge = verts[verts.len() - 1].bulge;
                    if let Some(seg) = bulge_to_arc(a, b, bulge) {
                        p.segments.push(seg);
                    }
                }

                if !p.segments.is_empty() {
                    model.entities.push(Entity::Path(p));
                }
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
                    text: collect_text(&g2),
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

                if lvl == SupportLevel::BestEffort {
                    for r in sm.reasons("dxf", "entity_spline", "import") {
                        warnings.push(AppError::new(r, "DXF SPLINE best-effort conversion"));
                    }
                }

                if !p.segments.is_empty() {
                    model.entities.push(Entity::Path(p));
                }
            }

            other => {
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_DXF_ENTITY_UNKNOWN_DROPPED,
                        "unknown DXF entity dropped",
                    )
                    .with_context("entity", other.to_string()),
                );
            }
        }
    }

    report.entities_in = model.entities.len();
    report.entities_out = model.entities.len();
    report.determinism_tag = opts.determinism_tag();
    Ok((model, warnings, report))
}

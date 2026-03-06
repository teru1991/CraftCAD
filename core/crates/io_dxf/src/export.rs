use craftcad_io::model::*;
use craftcad_io::options::ExportOptions;
use craftcad_io::reasons::{AppError, AppResult};
use craftcad_io::report::IoReport;
use craftcad_io_support::{MappingRules, SupportLevel, SupportMatrix};
use std::f64::consts::PI;

fn header_units(units: Units) -> i32 {
    match units {
        Units::Inch => 1,
        Units::Mm => 4,
    }
}

fn unit_scale(from: Units, to: Units) -> f64 {
    match (from, to) {
        (Units::Mm, Units::Inch) => 1.0 / 25.4,
        (Units::Inch, Units::Mm) => 25.4,
        _ => 1.0,
    }
}

fn fmt_fixed(v: f64, dp: usize) -> String {
    format!("{:.*}", dp, v)
}

fn rad_to_deg(r: f64) -> f64 {
    r * 180.0 / PI
}

fn norm_deg(mut d: f64) -> f64 {
    while d < 0.0 {
        d += 360.0;
    }
    while d >= 360.0 {
        d -= 360.0;
    }
    d
}

fn push_group(out: &mut String, code: i32, value: &str) {
    out.push_str(&format!("{}\n{}\n", code, value));
}

fn point_scaled(p: Point2D, s: f64) -> Point2D {
    Point2D {
        x: p.x * s,
        y: p.y * s,
    }
}

fn emit_line(out: &mut String, layer: &str, linetype: &str, a: Point2D, b: Point2D, dp: usize) {
    out.push_str("0\nLINE\n");
    push_group(out, 8, layer);
    push_group(out, 6, linetype);
    push_group(out, 10, &fmt_fixed(a.x, dp));
    push_group(out, 20, &fmt_fixed(a.y, dp));
    push_group(out, 11, &fmt_fixed(b.x, dp));
    push_group(out, 21, &fmt_fixed(b.y, dp));
}

fn emit_lwpolyline(
    out: &mut String,
    layer: &str,
    linetype: &str,
    points: &[Point2D],
    closed: bool,
    dp: usize,
) {
    out.push_str("0\nLWPOLYLINE\n");
    push_group(out, 8, layer);
    push_group(out, 6, linetype);
    push_group(out, 90, &format!("{}", points.len()));
    if closed {
        push_group(out, 70, "1");
    }
    for p in points {
        push_group(out, 10, &fmt_fixed(p.x, dp));
        push_group(out, 20, &fmt_fixed(p.y, dp));
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_arc(
    out: &mut String,
    layer: &str,
    linetype: &str,
    center: Point2D,
    radius: f64,
    start_rad: f64,
    end_rad: f64,
    ccw: bool,
    dp: usize,
) {
    out.push_str("0\nARC\n");
    push_group(out, 8, layer);
    push_group(out, 6, linetype);
    push_group(out, 10, &fmt_fixed(center.x, dp));
    push_group(out, 20, &fmt_fixed(center.y, dp));
    push_group(out, 40, &fmt_fixed(radius, dp));

    let (s, e) = if ccw {
        (start_rad, end_rad)
    } else {
        (end_rad, start_rad)
    };
    push_group(out, 50, &fmt_fixed(norm_deg(rad_to_deg(s)), dp));
    push_group(out, 51, &fmt_fixed(norm_deg(rad_to_deg(e)), dp));
}

fn emit_circle(
    out: &mut String,
    layer: &str,
    linetype: &str,
    center: Point2D,
    radius: f64,
    dp: usize,
) {
    out.push_str("0\nCIRCLE\n");
    push_group(out, 8, layer);
    push_group(out, 6, linetype);
    push_group(out, 10, &fmt_fixed(center.x, dp));
    push_group(out, 20, &fmt_fixed(center.y, dp));
    push_group(out, 40, &fmt_fixed(radius, dp));
}

#[allow(clippy::too_many_arguments)]
fn emit_text(
    out: &mut String,
    layer: &str,
    linetype: &str,
    pos: Point2D,
    size: f64,
    rot_deg: f64,
    text: &str,
    dp: usize,
) {
    out.push_str("0\nTEXT\n");
    push_group(out, 8, layer);
    push_group(out, 6, linetype);
    push_group(out, 10, &fmt_fixed(pos.x, dp));
    push_group(out, 20, &fmt_fixed(pos.y, dp));
    push_group(out, 40, &fmt_fixed(size, dp));
    push_group(out, 50, &fmt_fixed(norm_deg(rot_deg), dp));
    push_group(out, 1, text);
}

fn cubic_flatten_uniform(
    a: Point2D,
    c1: Point2D,
    c2: Point2D,
    b: Point2D,
    seg: usize,
) -> Vec<Point2D> {
    fn lerp(a: Point2D, b: Point2D, t: f64) -> Point2D {
        Point2D {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        }
    }
    fn cubic(a: Point2D, c1: Point2D, c2: Point2D, b: Point2D, t: f64) -> Point2D {
        let p0 = lerp(a, c1, t);
        let p1 = lerp(c1, c2, t);
        let p2 = lerp(c2, b, t);
        let q0 = lerp(p0, p1, t);
        let q1 = lerp(p1, p2, t);
        lerp(q0, q1, t)
    }

    let mut pts = Vec::with_capacity(seg + 1);
    for i in 0..=seg {
        let t = i as f64 / seg as f64;
        pts.push(cubic(a, c1, c2, b, t));
    }
    pts
}

pub fn export_dxf(
    model: &InternalModel,
    opts: &ExportOptions,
) -> AppResult<(Vec<u8>, Vec<AppError>, IoReport)> {
    let sm = SupportMatrix::load_from_ssot()?;
    let mr = MappingRules::load_from_ssot()?;

    let dp = mr.export_decimal_places() as usize;
    let scale = unit_scale(model.units, opts.target_units);

    let mut warnings = Vec::new();
    let mut report = IoReport::new("dxf");
    let mut out = String::new();

    out.push_str("0\nSECTION\n2\nHEADER\n9\n$INSUNITS\n70\n");
    out.push_str(&format!("{}\n", header_units(opts.target_units)));
    out.push_str("0\nENDSEC\n0\nSECTION\n2\nENTITIES\n");

    for e in &model.entities {
        match e {
            Entity::Path(p) => {
                let layer = mr.map_layer(&p.stroke.layer);
                let linetype = mr.map_linetype(&p.stroke.linetype);

                if p.segments.len() == 1 {
                    match p.segments[0] {
                        Segment2D::Line { a, b } => {
                            if sm.level("dxf", "entity_line", "export")
                                != SupportLevel::NotSupported
                            {
                                emit_line(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(a, scale),
                                    point_scaled(b, scale),
                                    dp,
                                );
                            }
                            continue;
                        }
                        Segment2D::Arc {
                            center,
                            radius,
                            start_rad,
                            end_rad,
                            ccw,
                        } => {
                            if sm.level("dxf", "entity_arc", "export") != SupportLevel::NotSupported
                            {
                                emit_arc(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(center, scale),
                                    radius * scale.abs(),
                                    start_rad,
                                    end_rad,
                                    ccw,
                                    dp,
                                );
                            }
                            continue;
                        }
                        Segment2D::Circle { center, radius } => {
                            if sm.level("dxf", "entity_circle", "export")
                                != SupportLevel::NotSupported
                            {
                                emit_circle(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(center, scale),
                                    radius * scale.abs(),
                                    dp,
                                );
                            }
                            continue;
                        }
                        Segment2D::CubicBezier { a, c1, c2, b } => {
                            let lvl = sm.level("dxf", "entity_cubic_bezier", "export");
                            let seg = opts
                                .determinism
                                .approx_min_segments
                                .max(8)
                                .min(opts.determinism.approx_max_segments.max(8));
                            let pts = cubic_flatten_uniform(a, c1, c2, b, seg)
                                .into_iter()
                                .map(|p| point_scaled(p, scale))
                                .collect::<Vec<_>>();

                            if lvl != SupportLevel::NotSupported {
                                for r in sm.reasons("dxf", "entity_cubic_bezier", "export") {
                                    warnings.push(
                                        AppError::new(
                                            r,
                                            "cubic bezier approximated for DXF export",
                                        )
                                        .with_context("path_id", p.id.clone())
                                        .with_context("segments", seg.to_string()),
                                    );
                                }
                                emit_lwpolyline(&mut out, &layer, &linetype, &pts, false, dp);
                            }
                            continue;
                        }
                    }
                }

                let all_line = p
                    .segments
                    .iter()
                    .all(|s| matches!(s, Segment2D::Line { .. }));
                if all_line {
                    if sm.level("dxf", "entity_polyline", "export") == SupportLevel::NotSupported {
                        continue;
                    }
                    let mut pts: Vec<Point2D> = Vec::new();
                    for (idx, s) in p.segments.iter().enumerate() {
                        if let Segment2D::Line { a, b } = s {
                            if idx == 0 {
                                pts.push(point_scaled(*a, scale));
                            }
                            pts.push(point_scaled(*b, scale));
                        }
                    }
                    if p.closed && pts.len() >= 2 {
                        let first = pts[0];
                        let last = pts[pts.len() - 1];
                        if (first.x - last.x).abs() <= 1e-12 && (first.y - last.y).abs() <= 1e-12 {
                            pts.pop();
                        }
                    }

                    if pts.len() >= 2 {
                        emit_lwpolyline(&mut out, &layer, &linetype, &pts, p.closed, dp);
                    }
                    continue;
                }

                for s in &p.segments {
                    match *s {
                        Segment2D::Line { a, b } => {
                            if sm.level("dxf", "entity_line", "export")
                                != SupportLevel::NotSupported
                            {
                                emit_line(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(a, scale),
                                    point_scaled(b, scale),
                                    dp,
                                );
                            }
                        }
                        Segment2D::Arc {
                            center,
                            radius,
                            start_rad,
                            end_rad,
                            ccw,
                        } => {
                            if sm.level("dxf", "entity_arc", "export") != SupportLevel::NotSupported
                            {
                                emit_arc(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(center, scale),
                                    radius * scale.abs(),
                                    start_rad,
                                    end_rad,
                                    ccw,
                                    dp,
                                );
                            }
                        }
                        Segment2D::Circle { center, radius } => {
                            if sm.level("dxf", "entity_circle", "export")
                                != SupportLevel::NotSupported
                            {
                                emit_circle(
                                    &mut out,
                                    &layer,
                                    &linetype,
                                    point_scaled(center, scale),
                                    radius * scale.abs(),
                                    dp,
                                );
                            }
                        }
                        Segment2D::CubicBezier { a, c1, c2, b } => {
                            let lvl = sm.level("dxf", "entity_path_unhandled_segment", "export");
                            if lvl == SupportLevel::NotSupported {
                                continue;
                            }
                            let seg = opts
                                .determinism
                                .approx_min_segments
                                .max(8)
                                .min(opts.determinism.approx_max_segments.max(8));
                            let pts = cubic_flatten_uniform(a, c1, c2, b, seg)
                                .into_iter()
                                .map(|p| point_scaled(p, scale))
                                .collect::<Vec<_>>();

                            for r in sm.reasons("dxf", "entity_path_unhandled_segment", "export") {
                                warnings.push(
                                    AppError::new(
                                        r,
                                        "unhandled segment approximated for DXF export",
                                    )
                                    .with_context("path_id", p.id.clone())
                                    .with_context("segments", seg.to_string()),
                                );
                            }
                            emit_lwpolyline(&mut out, &layer, &linetype, &pts, false, dp);
                        }
                    }
                }
            }

            Entity::Text(t) => {
                let lvl = sm.level("dxf", "entity_text", "export");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }

                let layer = mr.map_layer(&t.layer);
                let linetype = mr.map_linetype("CONTINUOUS");
                let pos = point_scaled(t.pos, scale);
                let size = (t.size as f64) * scale.abs();
                let rot_deg = rad_to_deg(t.rotation_rad);

                emit_text(&mut out, &layer, &linetype, pos, size, rot_deg, &t.text, dp);

                if lvl == SupportLevel::BestEffort {
                    for r in sm.reasons("dxf", "entity_text", "export") {
                        warnings.push(AppError::new(r, "DXF text exported best-effort"));
                    }
                }
            }
        }
    }

    out.push_str("0\nENDSEC\n0\nEOF\n");

    report.entities_in = model.entities.len();
    report.entities_out = model.entities.len();
    report.determinism_tag = opts.determinism_tag();

    Ok((out.into_bytes(), warnings, report))
}

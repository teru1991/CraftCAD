use crate::mapping::map_stroke;
use crate::parse::{parse_svg_dom, SvgDom, SvgNode};
use crate::pathdata::parse_path_segments;
use crate::transform::{parse_transform_attr, Affine2};
use craftcad_io::model::*;
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use craftcad_io_support::{MappingRules, SupportLevel, SupportMatrix};

fn attr<'a>(node: &'a SvgNode, key: &str) -> Option<&'a str> {
    node.attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn parse_f64(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

fn parse_points_list(s: &str) -> Vec<Point2D> {
    let cleaned = s.replace(',', " ");
    let nums: Vec<f64> = cleaned
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    nums.chunks(2)
        .filter_map(|w| {
            if w.len() == 2 {
                Some(Point2D { x: w[0], y: w[1] })
            } else {
                None
            }
        })
        .collect()
}

fn stroke_from_node(mr: &MappingRules, node: &SvgNode, fallback_layer: &str) -> StrokeStyle {
    let mut s = StrokeStyle::default();
    if let Some(cls) = attr(node, "class") {
        s.layer = cls.to_string();
    } else if let Some(id) = attr(node, "id") {
        s.layer = id.to_string();
    } else {
        s.layer = fallback_layer.to_string();
    }
    map_stroke(mr, s)
}

fn apply_tf_to_segment(tf: Affine2, s: Segment2D) -> Segment2D {
    match s {
        Segment2D::Line { a, b } => Segment2D::Line {
            a: tf.apply_point(a),
            b: tf.apply_point(b),
        },
        Segment2D::CubicBezier { a, c1, c2, b } => Segment2D::CubicBezier {
            a: tf.apply_point(a),
            c1: tf.apply_point(c1),
            c2: tf.apply_point(c2),
            b: tf.apply_point(b),
        },
        other => other,
    }
}

fn circle_to_segments(
    tf: Affine2,
    center: Point2D,
    radius: f64,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    sm: &SupportMatrix,
    path_id: &str,
) -> Vec<Segment2D> {
    let eps = opts.determinism.close_eps.max(1e-12);
    if let Some(s) = tf.approx_uniform_scale(eps) {
        let c = tf.apply_point(center);
        return vec![Segment2D::Circle {
            center: c,
            radius: radius * s,
        }];
    }

    let k = 0.552_284_749_830_793_6;
    let r = radius;
    let p0 = Point2D {
        x: center.x + r,
        y: center.y,
    };
    let p1 = Point2D {
        x: center.x,
        y: center.y + r,
    };
    let p2 = Point2D {
        x: center.x - r,
        y: center.y,
    };
    let p3 = Point2D {
        x: center.x,
        y: center.y - r,
    };

    let c01 = Point2D {
        x: p0.x,
        y: p0.y + k * r,
    };
    let c11 = Point2D {
        x: p1.x + k * r,
        y: p1.y,
    };

    let c12 = Point2D {
        x: p1.x - k * r,
        y: p1.y,
    };
    let c22 = Point2D {
        x: p2.x,
        y: p2.y + k * r,
    };

    let c23 = Point2D {
        x: p2.x,
        y: p2.y - k * r,
    };
    let c33 = Point2D {
        x: p3.x - k * r,
        y: p3.y,
    };

    let c30 = Point2D {
        x: p3.x + k * r,
        y: p3.y,
    };
    let c00 = Point2D {
        x: p0.x,
        y: p0.y - k * r,
    };

    let segs = vec![
        Segment2D::CubicBezier {
            a: p0,
            c1: c01,
            c2: c11,
            b: p1,
        },
        Segment2D::CubicBezier {
            a: p1,
            c1: c12,
            c2: c22,
            b: p2,
        },
        Segment2D::CubicBezier {
            a: p2,
            c1: c23,
            c2: c33,
            b: p3,
        },
        Segment2D::CubicBezier {
            a: p3,
            c1: c30,
            c2: c00,
            b: p0,
        },
    ];

    if sm.level("svg", "entity_circle", "import") == SupportLevel::BestEffort {
        for r in sm.reasons("svg", "entity_circle", "import") {
            warnings.push(
                AppError::new(r, "circle approximated to cubic under affine transform")
                    .with_context("id", path_id.to_string())
                    .with_context("method", "circle_cubic_4seg"),
            );
        }
    } else {
        warnings.push(
            AppError::new(
                ReasonCode::IO_CURVE_APPROX_APPLIED,
                "circle transformed by non-uniform affine; approximated by cubic",
            )
            .with_context("id", path_id.to_string())
            .with_context("method", "circle_cubic_4seg"),
        );
    }

    segs.into_iter()
        .map(|s| apply_tf_to_segment(tf, s))
        .collect()
}

fn walk_svg(
    node: &SvgNode,
    parent_tf: Affine2,
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
    sm: &SupportMatrix,
    mr: &MappingRules,
) {
    let _ = report;
    let local_tf = match parse_transform_attr(attr(node, "transform")) {
        Ok(m) => m,
        Err(e) => {
            warnings.push(e);
            Affine2::identity()
        }
    };
    let tf = parent_tf.mul(local_tf);

    if node.name == "image" || node.name == "use" {
        if sm.level("svg", "external_reference", "import") != SupportLevel::Supported {
            for r in sm.reasons("svg", "external_reference", "import") {
                warnings.push(AppError::new(r, "external reference dropped (image/use)"));
            }
        }
        return;
    }

    match node.name.as_str() {
        "g" | "svg" => {}
        "path" => {
            if sm.level("svg", "entity_path", "import") == SupportLevel::NotSupported {
                return;
            }
            let d = attr(node, "d").unwrap_or("");
            if d.is_empty() {
                return;
            }
            let stroke = stroke_from_node(mr, node, "0");
            let mut p = PathEntity::new(format!("svg_path_{}", model.entities.len()), stroke);

            let mut segs = parse_path_segments(d, opts, warnings, sm);
            segs = segs
                .into_iter()
                .map(|s| apply_tf_to_segment(tf, s))
                .collect();
            if segs.is_empty() {
                return;
            }
            p.segments = segs;
            model.entities.push(Entity::Path(p));
        }
        "line" => {
            if sm.level("svg", "entity_line", "import") == SupportLevel::NotSupported {
                return;
            }
            let x1 = attr(node, "x1").and_then(parse_f64).unwrap_or(0.0);
            let y1 = attr(node, "y1").and_then(parse_f64).unwrap_or(0.0);
            let x2 = attr(node, "x2").and_then(parse_f64).unwrap_or(0.0);
            let y2 = attr(node, "y2").and_then(parse_f64).unwrap_or(0.0);

            let a = tf.apply_point(Point2D { x: x1, y: y1 });
            let b = tf.apply_point(Point2D { x: x2, y: y2 });

            let mut p = PathEntity::new(
                format!("svg_line_{}", model.entities.len()),
                stroke_from_node(mr, node, "0"),
            );
            p.segments.push(Segment2D::Line { a, b });
            model.entities.push(Entity::Path(p));
        }
        "polyline" | "polygon" => {
            let feat = if node.name == "polygon" {
                "entity_polygon"
            } else {
                "entity_polyline"
            };
            if sm.level("svg", feat, "import") == SupportLevel::NotSupported {
                return;
            }
            let pts = attr(node, "points")
                .map(parse_points_list)
                .unwrap_or_default();
            if pts.len() < 2 {
                return;
            }

            let mut p = PathEntity::new(
                format!("svg_pl_{}", model.entities.len()),
                stroke_from_node(mr, node, "0"),
            );

            let tpts: Vec<Point2D> = pts.into_iter().map(|p| tf.apply_point(p)).collect();
            for w in tpts.windows(2) {
                p.segments.push(Segment2D::Line { a: w[0], b: w[1] });
            }
            if node.name == "polygon" {
                let first = tpts[0];
                let last = tpts[tpts.len() - 1];
                p.segments.push(Segment2D::Line { a: last, b: first });
            }
            model.entities.push(Entity::Path(p));
        }
        "circle" => {
            if sm.level("svg", "entity_circle", "import") == SupportLevel::NotSupported {
                return;
            }
            let cx = attr(node, "cx").and_then(parse_f64).unwrap_or(0.0);
            let cy = attr(node, "cy").and_then(parse_f64).unwrap_or(0.0);
            let r = attr(node, "r").and_then(parse_f64).unwrap_or(0.0);
            if r <= 0.0 {
                return;
            }

            let stroke = stroke_from_node(mr, node, "0");
            let id = format!("svg_circle_{}", model.entities.len());
            let segs = circle_to_segments(tf, Point2D { x: cx, y: cy }, r, opts, warnings, sm, &id);

            let mut p = PathEntity::new(id, stroke);
            p.segments = segs;
            model.entities.push(Entity::Path(p));
        }
        "text" => {
            let lvl = sm.level("svg", "entity_text", "import");
            if lvl == SupportLevel::NotSupported {
                return;
            }
            let x = attr(node, "x").and_then(parse_f64).unwrap_or(0.0);
            let y = attr(node, "y").and_then(parse_f64).unwrap_or(0.0);

            let pos = tf.apply_point(Point2D { x, y });
            let rot = tf.angle_rad();

            let sx = (tf.a * tf.a + tf.b * tf.b).sqrt();
            let sy = (tf.c * tf.c + tf.d * tf.d).sqrt();
            let scale = ((sx + sy) * 0.5).max(1e-9);

            let layer = if let Some(cls) = attr(node, "class") {
                mr.map_layer(cls)
            } else {
                mr.map_layer("GUIDE")
            };

            let t = TextEntity {
                id: format!("svg_text_{}", model.entities.len()),
                layer,
                pos,
                text: node.text.clone().unwrap_or_default(),
                size: (12.0 * scale) as f32,
                font_hint: attr(node, "font-family").map(ToString::to_string),
                rotation_rad: rot,
            };

            if lvl == SupportLevel::BestEffort {
                for r in sm.reasons("svg", "entity_text", "import") {
                    warnings.push(AppError::new(
                        r,
                        "svg text imported best-effort (font hint)",
                    ));
                }
            }
            model.entities.push(Entity::Text(t));
        }
        _ => {}
    }

    for c in &node.children {
        walk_svg(c, tf, model, opts, warnings, report, sm, mr);
    }
}

pub fn import_svg(
    bytes: &[u8],
    opts: &ImportOptions,
) -> AppResult<(InternalModel, Vec<AppError>, IoReport)> {
    let mut warnings = Vec::new();
    let mut report = IoReport::new("svg");
    let sm = SupportMatrix::load_from_ssot()?;
    let mr = MappingRules::load_from_ssot()?;

    let SvgDom {
        root,
        warnings: parse_warn,
    } = parse_svg_dom(bytes, opts)?;
    warnings.extend(parse_warn);

    let mut units = Units::Mm;
    if opts.allow_unit_guess {
        report.unit_guessed = true;
        warnings.push(
            AppError::new(
                ReasonCode::IO_UNIT_GUESSED,
                "units guessed from svg (default mm)",
            )
            .with_context("method", "default_mm"),
        );
    }
    units = crate::mapping::map_units(&mr, units);

    let mut model = InternalModel::new(units);
    model.metadata.source_format = "svg".to_string();
    model.metadata.determinism_tag = opts.determinism_tag();

    walk_svg(
        &root,
        Affine2::identity(),
        &mut model,
        opts,
        &mut warnings,
        &mut report,
        &sm,
        &mr,
    );

    report.entities_in = model.entities.len();
    report.texts_in = model.texts.len();
    Ok((model, warnings, report))
}

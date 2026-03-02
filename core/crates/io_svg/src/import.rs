use crate::mapping::map_stroke;
use crate::parse::{parse_svg_dom, SvgNode};
use craftcad_io::model::*;
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
use craftcad_io::report::IoReport;
use craftcad_io_support::{MappingRules, SupportLevel, SupportMatrix};

fn attr(node: &SvgNode, key: &str) -> Option<String> {
    node.attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
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

fn polyline_segments(points: &[Point2D]) -> Vec<Segment2D> {
    points
        .windows(2)
        .map(|w| Segment2D::Line { a: w[0], b: w[1] })
        .collect()
}

fn parse_path_d(d: &str) -> Vec<(char, Vec<f64>)> {
    let mut out = Vec::new();
    let mut cur_cmd: Option<char> = None;
    let mut cur_nums: Vec<f64> = Vec::new();
    let mut token = String::new();
    let flush_num = |t: &mut String, nums: &mut Vec<f64>| {
        if !t.is_empty() {
            if let Ok(v) = t.parse::<f64>() {
                nums.push(v);
            }
            t.clear();
        }
    };

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() {
            flush_num(&mut token, &mut cur_nums);
            if let Some(cmd) = cur_cmd {
                out.push((cmd, cur_nums.clone()));
                cur_nums.clear();
            }
            cur_cmd = Some(ch);
        } else if ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E') {
            token.push(ch);
        } else {
            flush_num(&mut token, &mut cur_nums);
        }
    }
    flush_num(&mut token, &mut cur_nums);
    if let Some(cmd) = cur_cmd {
        out.push((cmd, cur_nums));
    }
    out
}

fn build_path_segments_from_svg(
    d: &str,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    sm: &SupportMatrix,
) -> Vec<Segment2D> {
    let mut segs = Vec::new();
    let cmds = parse_path_d(d);
    let mut cur = Point2D { x: 0.0, y: 0.0 };
    let mut start = cur;

    for (cmd, nums) in cmds {
        match cmd {
            'M' | 'm' => {
                if nums.len() >= 2 {
                    cur = Point2D {
                        x: nums[0],
                        y: nums[1],
                    };
                    start = cur;
                }
            }
            'L' | 'l' => {
                for w in nums.chunks(2) {
                    if w.len() == 2 {
                        let p = Point2D { x: w[0], y: w[1] };
                        segs.push(Segment2D::Line { a: cur, b: p });
                        cur = p;
                    }
                }
            }
            'C' | 'c' => {
                for w in nums.chunks(6) {
                    if w.len() == 6 {
                        let c1 = Point2D { x: w[0], y: w[1] };
                        let c2 = Point2D { x: w[2], y: w[3] };
                        let p = Point2D { x: w[4], y: w[5] };
                        segs.push(Segment2D::CubicBezier {
                            a: cur,
                            c1,
                            c2,
                            b: p,
                        });
                        cur = p;
                    }
                }
            }
            'A' | 'a' => {
                let lvl = sm.level("svg", "entity_path_elliptical_arc", "import");
                for w in nums.chunks(7) {
                    if w.len() == 7 {
                        let p = Point2D { x: w[5], y: w[6] };
                        let seg_count = opts.determinism.approx_min_segments.max(2);
                        for i in 1..=seg_count {
                            let t0 = (i - 1) as f64 / seg_count as f64;
                            let t1 = i as f64 / seg_count as f64;
                            let a = Point2D {
                                x: cur.x + (p.x - cur.x) * t0,
                                y: cur.y + (p.y - cur.y) * t0,
                            };
                            let b = Point2D {
                                x: cur.x + (p.x - cur.x) * t1,
                                y: cur.y + (p.y - cur.y) * t1,
                            };
                            segs.push(Segment2D::Line { a, b });
                        }
                        cur = p;
                        if lvl != SupportLevel::Supported {
                            warnings.push(
                                AppError::new(
                                    ReasonCode::IO_SVG_ARC_CONVERTED,
                                    "svg elliptical arc converted with deterministic polyline",
                                )
                                .with_context("method", "deterministic_polyline")
                                .with_context("segments", seg_count.to_string()),
                            );
                            for r in sm.reasons("svg", "entity_path_elliptical_arc", "import") {
                                warnings.push(AppError::new(r, "svg elliptical arc best-effort"));
                            }
                        }
                    }
                }
            }
            'Z' | 'z' => {
                segs.push(Segment2D::Line { a: cur, b: start });
                cur = start;
            }
            _ => warnings.push(
                AppError::new(
                    ReasonCode::IO_SVG_PATH_COMMAND_UNKNOWN,
                    "unknown svg path command; ignored",
                )
                .with_context("cmd", cmd.to_string()),
            ),
        }
    }
    segs
}

fn collect<'a>(root: &'a SvgNode, out: &mut Vec<&'a SvgNode>) {
    out.push(root);
    for c in &root.children {
        collect(c, out);
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
    let root = parse_svg_dom(bytes, opts)?;

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

    let mut nodes = Vec::new();
    collect(&root, &mut nodes);
    for node in nodes {
        match node.name.as_str() {
            "path" => {
                if sm.level("svg", "entity_path", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let d = attr(node, "d").unwrap_or_default();
                if d.is_empty() {
                    continue;
                }
                let mut stroke = StrokeStyle::default();
                if let Some(cls) = attr(node, "class") {
                    stroke.layer = cls;
                }
                stroke = map_stroke(&mr, stroke);
                let mut p = PathEntity::new(format!("svg_path_{}", model.entities.len()), stroke);
                p.segments = build_path_segments_from_svg(&d, opts, &mut warnings, &sm);
                model.entities.push(Entity::Path(p));
            }
            "line" => {
                if sm.level("svg", "entity_line", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let x1 = attr(node, "x1").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let y1 = attr(node, "y1").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let x2 = attr(node, "x2").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let y2 = attr(node, "y2").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let mut p = PathEntity::new(
                    format!("svg_line_{}", model.entities.len()),
                    map_stroke(&mr, StrokeStyle::default()),
                );
                p.segments.push(Segment2D::Line {
                    a: Point2D { x: x1, y: y1 },
                    b: Point2D { x: x2, y: y2 },
                });
                model.entities.push(Entity::Path(p));
            }
            "polyline" | "polygon" => {
                let feat = if node.name == "polygon" {
                    "entity_polygon"
                } else {
                    "entity_polyline"
                };
                if sm.level("svg", feat, "import") == SupportLevel::NotSupported {
                    continue;
                }
                let pts = attr(node, "points")
                    .map(|s| parse_points_list(&s))
                    .unwrap_or_default();
                if pts.len() < 2 {
                    continue;
                }
                let mut p = PathEntity::new(
                    format!("svg_pl_{}", model.entities.len()),
                    map_stroke(&mr, StrokeStyle::default()),
                );
                p.segments = polyline_segments(&pts);
                if node.name == "polygon" {
                    p.segments.push(Segment2D::Line {
                        a: *pts.last().unwrap(),
                        b: pts[0],
                    });
                }
                model.entities.push(Entity::Path(p));
            }
            "circle" => {
                if sm.level("svg", "entity_circle", "import") == SupportLevel::NotSupported {
                    continue;
                }
                let cx = attr(node, "cx").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let cy = attr(node, "cy").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let r = attr(node, "r").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let mut p = PathEntity::new(
                    format!("svg_circle_{}", model.entities.len()),
                    map_stroke(&mr, StrokeStyle::default()),
                );
                p.segments.push(Segment2D::Circle {
                    center: Point2D { x: cx, y: cy },
                    radius: r,
                });
                model.entities.push(Entity::Path(p));
            }
            "text" => {
                let lvl = sm.level("svg", "entity_text", "import");
                if lvl == SupportLevel::NotSupported {
                    continue;
                }
                let x = attr(node, "x").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let y = attr(node, "y").and_then(|s| parse_f64(&s)).unwrap_or(0.0);
                let t = TextEntity {
                    id: format!("svg_text_{}", model.entities.len()),
                    layer: mr.map_layer("GUIDE"),
                    pos: Point2D { x, y },
                    text: node.text.clone().unwrap_or_default(),
                    size: 12.0,
                    font_hint: attr(node, "font-family"),
                    rotation_rad: 0.0,
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
    }

    report.entities_in = model.entities.len();
    report.texts_in = model.texts.len();
    Ok((model, warnings, report))
}

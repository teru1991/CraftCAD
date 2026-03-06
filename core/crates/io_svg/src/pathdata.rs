use craftcad_io::model::{Point2D, Segment2D};
use craftcad_io::options::ImportOptions;
use craftcad_io::reasons::{AppError, ReasonCode};
use craftcad_io_support::{SupportLevel, SupportMatrix};

#[derive(Debug, Clone, Copy)]
enum Tok {
    Cmd(char),
    Num(f64),
}

fn tokenize(d: &str) -> Vec<Tok> {
    let bytes = d.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch.is_ascii_alphabetic() {
            out.push(Tok::Cmd(ch));
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let c = bytes[i] as char;
                if c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E') {
                    i += 1;
                } else {
                    break;
                }
            }
            if let Ok(v) = d[start..i].parse::<f64>() {
                out.push(Tok::Num(v));
            }
            continue;
        }
        i += 1;
    }
    out
}

fn reflect(p: Point2D, about: Point2D) -> Point2D {
    Point2D {
        x: about.x * 2.0 - p.x,
        y: about.y * 2.0 - p.y,
    }
}

fn quad_to_cubic(a: Point2D, q: Point2D, b: Point2D) -> Segment2D {
    let c1 = Point2D {
        x: a.x + (2.0 / 3.0) * (q.x - a.x),
        y: a.y + (2.0 / 3.0) * (q.y - a.y),
    };
    let c2 = Point2D {
        x: b.x + (2.0 / 3.0) * (q.x - b.x),
        y: b.y + (2.0 / 3.0) * (q.y - b.y),
    };
    Segment2D::CubicBezier { a, c1, c2, b }
}

fn angle_between(u: (f64, f64), v: (f64, f64)) -> f64 {
    let (ux, uy) = u;
    let (vx, vy) = v;
    let dot = ux * vx + uy * vy;
    let det = ux * vy - uy * vx;
    det.atan2(dot)
}

fn arc_to_cubics(
    p0: Point2D,
    p1: Point2D,
    mut rx: f64,
    mut ry: f64,
    x_axis_rotation_deg: f64,
    large_arc: bool,
    sweep: bool,
) -> Vec<Segment2D> {
    if rx == 0.0 || ry == 0.0 || (p0.x == p1.x && p0.y == p1.y) {
        return vec![Segment2D::Line { a: p0, b: p1 }];
    }

    rx = rx.abs();
    ry = ry.abs();
    let phi = x_axis_rotation_deg.to_radians();
    let (sin_phi, cos_phi) = phi.sin_cos();

    let dx2 = (p0.x - p1.x) * 0.5;
    let dy2 = (p0.y - p1.y) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let lam = (x1p2 / rx2) + (y1p2 / ry2);
    if lam > 1.0 {
        let s = lam.sqrt();
        rx *= s;
        ry *= s;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;

    let num = (rx2 * ry2) - (rx2 * y1p2) - (ry2 * x1p2);
    let den = (rx2 * y1p2) + (ry2 * x1p2);
    let mut coef = 0.0;
    if den != 0.0 {
        let sign = if large_arc == sweep { -1.0 } else { 1.0 };
        coef = sign * (num / den).max(0.0).sqrt();
    }
    let cxp = coef * (rx * y1p / ry);
    let cyp = coef * (-ry * x1p / rx);

    let cx = cos_phi * cxp - sin_phi * cyp + (p0.x + p1.x) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (p0.y + p1.y) * 0.5;

    let v1 = ((x1p - cxp) / rx, (y1p - cyp) / ry);
    let v2 = ((-x1p - cxp) / rx, (-y1p - cyp) / ry);

    let theta1 = angle_between((1.0, 0.0), v1);
    let mut dtheta = angle_between(v1, v2);

    if !sweep && dtheta > 0.0 {
        dtheta -= std::f64::consts::TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += std::f64::consts::TAU;
    }

    let segs = (dtheta.abs() / (std::f64::consts::FRAC_PI_2))
        .ceil()
        .max(1.0) as usize;
    let delta = dtheta / segs as f64;

    let mut out = Vec::with_capacity(segs);
    let mut cur = p0;

    for i in 0..segs {
        let t0 = theta1 + i as f64 * delta;
        let t1 = t0 + delta;

        let (s0, c0) = t0.sin_cos();
        let (s1, c1) = t1.sin_cos();

        let p0u = (c0, s0);
        let p1u = (c1, s1);

        let alpha = (4.0 / 3.0) * ((t1 - t0) / 4.0).tan();

        let c1u = (p0u.0 - alpha * p0u.1, p0u.1 + alpha * p0u.0);
        let c2u = (p1u.0 + alpha * p1u.1, p1u.1 - alpha * p1u.0);

        let to_pt = |u: (f64, f64)| -> Point2D {
            let x = u.0 * rx;
            let y = u.1 * ry;
            Point2D {
                x: cx + cos_phi * x - sin_phi * y,
                y: cy + sin_phi * x + cos_phi * y,
            }
        };

        let b = to_pt(p1u);
        let cc1 = to_pt(c1u);
        let cc2 = to_pt(c2u);

        out.push(Segment2D::CubicBezier {
            a: cur,
            c1: cc1,
            c2: cc2,
            b,
        });
        cur = b;
    }

    out
}

pub fn parse_path_segments(
    d: &str,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    sm: &SupportMatrix,
) -> Vec<Segment2D> {
    let toks = tokenize(d);
    if toks.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<Segment2D> = Vec::new();
    let mut i = 0usize;
    let mut cmd: Option<char> = None;

    let mut cur = Point2D { x: 0.0, y: 0.0 };
    let mut sub_start = cur;

    let mut prev_cubic_c2: Option<Point2D> = None;
    let mut prev_quad_q: Option<Point2D> = None;

    let next_num = |toks: &[Tok], i: &mut usize| -> Option<f64> {
        if *i < toks.len() {
            match toks[*i] {
                Tok::Num(v) => {
                    *i += 1;
                    Some(v)
                }
                Tok::Cmd(_) => None,
            }
        } else {
            None
        }
    };

    while i < toks.len() {
        match toks[i] {
            Tok::Cmd(c) => {
                cmd = Some(c);
                i += 1;
            }
            Tok::Num(_) => {
                if cmd.is_none() {
                    warnings.push(
                        AppError::new(
                            ReasonCode::IO_PARSE_SVG_MALFORMED,
                            "path data starts with number; ignored",
                        )
                        .with_context("d_prefix", d.chars().take(32).collect::<String>()),
                    );
                    break;
                }
            }
        }

        let Some(c) = cmd else { break };
        match c {
            'M' | 'm' => {
                let mut first = true;
                while let (Some(x), Some(y)) = (next_num(&toks, &mut i), next_num(&toks, &mut i)) {
                    let prev = cur;
                    let p = if c == 'm' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    cur = p;
                    if first {
                        sub_start = p;
                        first = false;
                    } else {
                        out.push(Segment2D::Line { a: prev, b: p });
                    }
                    prev_cubic_c2 = None;
                    prev_quad_q = None;
                    cmd = Some(if c == 'm' { 'l' } else { 'L' });
                }
            }
            'L' | 'l' => {
                while let (Some(x), Some(y)) = (next_num(&toks, &mut i), next_num(&toks, &mut i)) {
                    let p = if c == 'l' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    out.push(Segment2D::Line { a: cur, b: p });
                    cur = p;
                    prev_cubic_c2 = None;
                    prev_quad_q = None;
                }
            }
            'H' | 'h' => {
                while let Some(x) = next_num(&toks, &mut i) {
                    let p = if c == 'h' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y,
                        }
                    } else {
                        Point2D { x, y: cur.y }
                    };
                    out.push(Segment2D::Line { a: cur, b: p });
                    cur = p;
                    prev_cubic_c2 = None;
                    prev_quad_q = None;
                }
            }
            'V' | 'v' => {
                while let Some(y) = next_num(&toks, &mut i) {
                    let p = if c == 'v' {
                        Point2D {
                            x: cur.x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x: cur.x, y }
                    };
                    out.push(Segment2D::Line { a: cur, b: p });
                    cur = p;
                    prev_cubic_c2 = None;
                    prev_quad_q = None;
                }
            }
            'C' | 'c' => {
                while let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                ) {
                    let c1 = if c == 'c' {
                        Point2D {
                            x: cur.x + x1,
                            y: cur.y + y1,
                        }
                    } else {
                        Point2D { x: x1, y: y1 }
                    };
                    let c2 = if c == 'c' {
                        Point2D {
                            x: cur.x + x2,
                            y: cur.y + y2,
                        }
                    } else {
                        Point2D { x: x2, y: y2 }
                    };
                    let p = if c == 'c' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    out.push(Segment2D::CubicBezier {
                        a: cur,
                        c1,
                        c2,
                        b: p,
                    });
                    prev_cubic_c2 = Some(c2);
                    prev_quad_q = None;
                    cur = p;
                }
            }
            'S' | 's' => {
                while let (Some(x2), Some(y2), Some(x), Some(y)) = (
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                ) {
                    let c1 = if let Some(prev) = prev_cubic_c2 {
                        reflect(prev, cur)
                    } else {
                        cur
                    };
                    let c2 = if c == 's' {
                        Point2D {
                            x: cur.x + x2,
                            y: cur.y + y2,
                        }
                    } else {
                        Point2D { x: x2, y: y2 }
                    };
                    let p = if c == 's' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    out.push(Segment2D::CubicBezier {
                        a: cur,
                        c1,
                        c2,
                        b: p,
                    });
                    prev_cubic_c2 = Some(c2);
                    prev_quad_q = None;
                    cur = p;
                }
            }
            'Q' | 'q' => {
                while let (Some(x1), Some(y1), Some(x), Some(y)) = (
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                ) {
                    let q = if c == 'q' {
                        Point2D {
                            x: cur.x + x1,
                            y: cur.y + y1,
                        }
                    } else {
                        Point2D { x: x1, y: y1 }
                    };
                    let p = if c == 'q' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    out.push(quad_to_cubic(cur, q, p));
                    prev_quad_q = Some(q);
                    prev_cubic_c2 = None;
                    cur = p;
                }
            }
            'T' | 't' => {
                while let (Some(x), Some(y)) = (next_num(&toks, &mut i), next_num(&toks, &mut i)) {
                    let q = if let Some(prev) = prev_quad_q {
                        reflect(prev, cur)
                    } else {
                        cur
                    };
                    let p = if c == 't' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };
                    out.push(quad_to_cubic(cur, q, p));
                    prev_quad_q = Some(q);
                    prev_cubic_c2 = None;
                    cur = p;
                }
            }
            'A' | 'a' => {
                let lvl = sm.level("svg", "entity_path_elliptical_arc", "import");
                while let (
                    Some(rx),
                    Some(ry),
                    Some(rot),
                    Some(large),
                    Some(sweep),
                    Some(x),
                    Some(y),
                ) = (
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                    next_num(&toks, &mut i),
                ) {
                    let p = if c == 'a' {
                        Point2D {
                            x: cur.x + x,
                            y: cur.y + y,
                        }
                    } else {
                        Point2D { x, y }
                    };

                    let segs = arc_to_cubics(
                        cur,
                        p,
                        rx,
                        ry,
                        rot,
                        (large.round() as i32) != 0,
                        (sweep.round() as i32) != 0,
                    );

                    if lvl == SupportLevel::BestEffort {
                        for r in sm.reasons("svg", "entity_path_elliptical_arc", "import") {
                            warnings.push(
                                AppError::new(
                                    r,
                                    "svg arc converted to cubic beziers (best-effort)",
                                )
                                .with_context("method", "arc_to_cubic")
                                .with_context("segments", segs.len().to_string()),
                            );
                        }
                    }

                    out.extend(segs);
                    prev_cubic_c2 = None;
                    prev_quad_q = None;
                    cur = p;
                }
            }
            'Z' | 'z' => {
                out.push(Segment2D::Line {
                    a: cur,
                    b: sub_start,
                });
                cur = sub_start;
                prev_cubic_c2 = None;
                prev_quad_q = None;
            }
            _ => {
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_SVG_PATH_COMMAND_UNKNOWN,
                        "unknown svg path command; ignored",
                    )
                    .with_context("cmd", c.to_string()),
                );
                while i < toks.len() {
                    if matches!(toks[i], Tok::Cmd(_)) {
                        break;
                    }
                    i += 1;
                }
            }
        }
    }

    out.retain(Segment2D::is_finite);
    let _ = opts;
    out
}

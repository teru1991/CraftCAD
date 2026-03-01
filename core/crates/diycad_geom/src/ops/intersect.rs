use crate::{util::dist2, EpsilonPolicy, Geom2D, IntersectionSet, Vec2};
use craftcad_serialize::{Reason, ReasonCode, Result};

fn normalize_angle(mut a: f64) -> f64 {
    while a <= -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    while a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    }
    a
}

fn arc_sweep(start: f64, end: f64, ccw: bool) -> f64 {
    let s = normalize_angle(start);
    let e = normalize_angle(end);
    if ccw {
        let mut d = e - s;
        if d < 0.0 {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    } else {
        let mut d = s - e;
        if d < 0.0 {
            d += 2.0 * std::f64::consts::PI;
        }
        d
    }
}

fn in_arc_range(theta: f64, start: f64, end: f64, ccw: bool, eps: f64) -> bool {
    let t = normalize_angle(theta);
    let s = normalize_angle(start);
    if ccw {
        let mut u = t - s;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        u >= -eps && u <= arc_sweep(start, end, ccw) + eps
    } else {
        let mut u = s - t;
        if u < 0.0 {
            u += 2.0 * std::f64::consts::PI;
        }
        u >= -eps && u <= arc_sweep(start, end, ccw) + eps
    }
}

fn dedupe(mut pts: Vec<Vec2>, eps: &EpsilonPolicy) -> Vec<Vec2> {
    pts.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    let mut out = Vec::with_capacity(pts.len());
    for p in pts {
        if !out
            .iter()
            .any(|q| dist2(*q, p) <= eps.eq_dist * eps.eq_dist)
        {
            out.push(p);
        }
    }
    out
}

fn line_line(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2, eps: &EpsilonPolicy) -> Result<Vec<Vec2>> {
    let r = Vec2 {
        x: a1.x - a0.x,
        y: a1.y - a0.y,
    };
    let s = Vec2 {
        x: b1.x - b0.x,
        y: b1.y - b0.y,
    };
    let denom = r.x * s.y - r.y * s.x;
    let qp = Vec2 {
        x: b0.x - a0.x,
        y: b0.y - a0.y,
    };
    let qpxr = qp.x * r.y - qp.y * r.x;
    if denom.abs() <= eps.intersect_tol {
        if qpxr.abs() <= eps.intersect_tol {
            let mut reason = Reason::from_code(ReasonCode::GeomIntersectionAmbiguous);
            reason
                .debug
                .insert("case".into(), serde_json::json!("colinear_overlap"));
            return Err(reason);
        }
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }
    let t = (qp.x * s.y - qp.y * s.x) / denom;
    let u = (qp.x * r.y - qp.y * r.x) / denom;
    if t < -eps.intersect_tol
        || t > 1.0 + eps.intersect_tol
        || u < -eps.intersect_tol
        || u > 1.0 + eps.intersect_tol
    {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }
    Ok(vec![Vec2 {
        x: a0.x + t * r.x,
        y: a0.y + t * r.y,
    }])
}

fn line_circle(a: Vec2, b: Vec2, c: Vec2, r: f64, eps: &EpsilonPolicy) -> Result<Vec<Vec2>> {
    if !r.is_finite() || r <= eps.eq_dist {
        return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
    }
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let aa = dx * dx + dy * dy;
    if aa <= eps.eq_dist * eps.eq_dist {
        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
    }

    let fx = a.x - c.x;
    let fy = a.y - c.y;
    let bb = 2.0 * (fx * dx + fy * dy);
    let cc = fx * fx + fy * fy - r * r;
    let disc = bb * bb - 4.0 * aa * cc;

    if disc < -eps.intersect_tol {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }

    let sqrt_disc = if disc < 0.0 { 0.0 } else { disc.sqrt() };
    let mut pts = vec![];
    let s0 = (-bb - sqrt_disc) / (2.0 * aa);
    let s1 = (-bb + sqrt_disc) / (2.0 * aa);
    for s in [s0, s1] {
        if s >= -eps.intersect_tol && s <= 1.0 + eps.intersect_tol {
            pts.push(Vec2 {
                x: a.x + s * dx,
                y: a.y + s * dy,
            });
        }
    }

    let pts = dedupe(pts, eps);
    if pts.is_empty() {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }
    Ok(pts)
}

fn circle_circle(c0: Vec2, r0: f64, c1: Vec2, r1: f64, eps: &EpsilonPolicy) -> Result<Vec<Vec2>> {
    if !r0.is_finite() || !r1.is_finite() || r0 <= eps.eq_dist || r1 <= eps.eq_dist {
        return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
    }

    let dx = c1.x - c0.x;
    let dy = c1.y - c0.y;
    let d = (dx * dx + dy * dy).sqrt();

    if d <= eps.eq_dist && (r0 - r1).abs() <= eps.eq_dist {
        let mut reason = Reason::from_code(ReasonCode::GeomIntersectionAmbiguous);
        reason
            .debug
            .insert("case".into(), serde_json::json!("coincident_circles"));
        return Err(reason);
    }

    if d > r0 + r1 + eps.intersect_tol || d < (r0 - r1).abs() - eps.intersect_tol {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }

    let denom = d.max(eps.eq_dist);
    let a = (r0 * r0 - r1 * r1 + d * d) / (2.0 * denom);
    let h2 = r0 * r0 - a * a;
    if h2 < -eps.intersect_tol {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }
    let h = if h2 < 0.0 { 0.0 } else { h2.sqrt() };

    let xm = c0.x + a * dx / denom;
    let ym = c0.y + a * dy / denom;
    let rx = -dy * (h / denom);
    let ry = dx * (h / denom);

    let pts = dedupe(
        vec![
            Vec2 {
                x: xm + rx,
                y: ym + ry,
            },
            Vec2 {
                x: xm - rx,
                y: ym - ry,
            },
        ],
        eps,
    );
    if pts.is_empty() {
        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
    }
    Ok(pts)
}

fn filter_points_for_arc(
    points: Vec<Vec2>,
    c: Vec2,
    start_angle: f64,
    end_angle: f64,
    ccw: bool,
    eps: &EpsilonPolicy,
) -> Result<Vec<Vec2>> {
    if !start_angle.is_finite() || !end_angle.is_finite() {
        return Err(Reason::from_code(ReasonCode::GeomArcRangeInvalid));
    }
    Ok(points
        .into_iter()
        .filter(|p| {
            let theta = (p.y - c.y).atan2(p.x - c.x);
            in_arc_range(theta, start_angle, end_angle, ccw, eps.intersect_tol)
        })
        .collect())
}

fn compute_with_eps(a: &Geom2D, b: &Geom2D, eps: &EpsilonPolicy) -> Result<Vec<Vec2>> {
    match (a, b) {
        (Geom2D::Line { a: a0, b: a1 }, Geom2D::Line { a: b0, b: b1 }) => {
            line_line(*a0, *a1, *b0, *b1, eps)
        }
        (Geom2D::Line { a, b }, Geom2D::Circle { c, r })
        | (Geom2D::Circle { c, r }, Geom2D::Line { a, b }) => line_circle(*a, *b, *c, *r, eps),
        (
            Geom2D::Line { a, b },
            Geom2D::Arc {
                c,
                r,
                start_angle,
                end_angle,
                ccw,
            },
        )
        | (
            Geom2D::Arc {
                c,
                r,
                start_angle,
                end_angle,
                ccw,
            },
            Geom2D::Line { a, b },
        ) => {
            let points = line_circle(*a, *b, *c, *r, eps)?;
            filter_points_for_arc(points, *c, *start_angle, *end_angle, *ccw, eps)
        }
        (Geom2D::Circle { c: c0, r: r0 }, Geom2D::Circle { c: c1, r: r1 }) => {
            circle_circle(*c0, *r0, *c1, *r1, eps)
        }
        (
            Geom2D::Polyline { pts, closed },
            Geom2D::Line {
                a: line_a,
                b: line_b,
            },
        )
        | (
            Geom2D::Line {
                a: line_a,
                b: line_b,
            },
            Geom2D::Polyline { pts, closed },
        ) => {
            let seg_count = if *closed {
                pts.len()
            } else {
                pts.len().saturating_sub(1)
            };
            let mut out = Vec::new();
            for i in 0..seg_count {
                let p0 = pts[i];
                let p1 = if i + 1 < pts.len() {
                    pts[i + 1]
                } else {
                    pts[0]
                };
                if let Ok(hit) = line_line(p0, p1, *line_a, *line_b, eps) {
                    out.extend(hit);
                }
            }
            Ok(dedupe(out, eps))
        }
        _ => {
            let mut reason = Reason::from_code(ReasonCode::GeomNoIntersection);
            reason
                .debug
                .insert("unsupported_pair".into(), serde_json::json!(true));
            Err(reason)
        }
    }
}

pub fn intersect(a: &Geom2D, b: &Geom2D, eps: &EpsilonPolicy) -> Result<IntersectionSet> {
    let mut fallback_info: Vec<String> = vec![];
    let mut tol = eps.intersect_tol.max(1e-12);
    let max_attempts = 3usize;
    let mut last_error: Option<Reason> = None;

    for attempt in 0..max_attempts {
        let mut try_eps = *eps;
        try_eps.intersect_tol = tol;
        match compute_with_eps(a, b, &try_eps) {
            Ok(points) => {
                let pts = dedupe(points, eps);
                if pts.is_empty() {
                    last_error = Some(Reason::from_code(ReasonCode::GeomNoIntersection));
                } else {
                    let truncated = pts.len() > 16;
                    let mut debug = serde_json::json!({
                        "classification": if pts.len() == 1 { "tangent_or_single" } else { "secant" },
                        "candidate_count": pts.len(),
                        "candidates": pts.iter().take(16).collect::<Vec<_>>(),
                        "truncated": truncated,
                    });
                    if !fallback_info.is_empty() {
                        debug["info"] = serde_json::json!(fallback_info);
                    }
                    return Ok(IntersectionSet {
                        ambiguous: pts.len() > 1,
                        points: pts,
                        debug,
                    });
                }
            }
            Err(reason) => {
                if reason.code == ReasonCode::GeomNoIntersection.as_str() {
                    last_error = Some(reason);
                } else {
                    return Err(reason);
                }
            }
        }

        if attempt + 1 < max_attempts {
            tol *= 10.0;
            fallback_info.push("GEOM_NUMERIC_UNSTABLE_FALLBACK_USED".to_string());
            continue;
        }
    }

    if fallback_info.is_empty() {
        Err(last_error.unwrap_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection)))
    } else if let Some(last) = last_error {
        if last.code == ReasonCode::GeomNoIntersection.as_str() {
            Err(last)
        } else {
            Err(Reason::from_code(ReasonCode::GeomFallbackLimitReached))
        }
    } else {
        Err(Reason::from_code(ReasonCode::GeomFallbackLimitReached))
    }
}

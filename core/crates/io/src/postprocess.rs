use crate::model::*;
use crate::options::{ImportOptions, OriginPolicy};
use crate::reasons::{AppError, ReasonCode};
use crate::report::IoReport;

fn dist(a: Point2D, b: Point2D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn path_endpoints(p: &PathEntity) -> Option<(Point2D, Point2D)> {
    let first = p.segments.first()?;
    let last = p.segments.last()?;

    let a = match first {
        Segment2D::Line { a, .. } => *a,
        Segment2D::CubicBezier { a, .. } => *a,
        _ => return None,
    };
    let b = match last {
        Segment2D::Line { b, .. } => *b,
        Segment2D::CubicBezier { b, .. } => *b,
        _ => return None,
    };
    Some((a, b))
}

fn translate_point(p: &mut Point2D, dx: f64, dy: f64) {
    p.x += dx;
    p.y += dy;
}

fn translate_segment(s: &mut Segment2D, dx: f64, dy: f64) {
    match s {
        Segment2D::Line { a, b } => {
            translate_point(a, dx, dy);
            translate_point(b, dx, dy);
        }
        Segment2D::Arc { center, .. } => {
            translate_point(center, dx, dy);
        }
        Segment2D::Circle { center, .. } => {
            translate_point(center, dx, dy);
        }
        Segment2D::CubicBezier { a, c1, c2, b } => {
            translate_point(a, dx, dy);
            translate_point(c1, dx, dy);
            translate_point(c2, dx, dy);
            translate_point(b, dx, dy);
        }
    }
}

fn bbox_of_model(model: &InternalModel) -> Option<BBox2D> {
    let mut bb = BBox2D::empty();
    let mut any = false;

    for e in &model.entities {
        let eb = e.bbox();
        if eb.is_valid() {
            bb.expand(eb.min);
            bb.expand(eb.max);
            any = true;
        }
    }

    if any && bb.is_valid() {
        Some(bb)
    } else {
        None
    }
}

fn tiny_len(seg: &Segment2D) -> Option<f64> {
    match seg {
        Segment2D::Line { a, b } => Some(dist(*a, *b)),
        Segment2D::CubicBezier { a, b, .. } => Some(dist(*a, *b)),
        _ => None,
    }
}

pub fn apply_origin_policy(
    model: &mut InternalModel,
    origin_policy: &OriginPolicy,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    if *origin_policy == OriginPolicy::MoveToZero {
        if let Some(bb) = bbox_of_model(model) {
            let dx = -bb.min.x;
            let dy = -bb.min.y;

            if dx != 0.0 || dy != 0.0 {
                for e in &mut model.entities {
                    if let Entity::Path(p) = e {
                        for s in &mut p.segments {
                            translate_segment(s, dx, dy);
                        }
                    } else if let Entity::Text(t) = e {
                        translate_point(&mut t.pos, dx, dy);
                    }
                }
                for t in &mut model.texts {
                    translate_point(&mut t.pos, dx, dy);
                }

                report.origin_shifted = true;
                warnings.push(
                    AppError::new(
                        ReasonCode::IO_ORIGIN_SHIFTED,
                        "origin shifted to (0,0) by bbox.min",
                    )
                    .with_context("dx", dx.to_string())
                    .with_context("dy", dy.to_string()),
                );
            }
        }
    }
}

pub fn remove_tiny_segments(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    let min_len = opts.determinism.tiny_segment_min_len.max(0.0);

    for e in &mut model.entities {
        if let Entity::Path(p) = e {
            let before = p.segments.len();
            p.segments.retain(|s| {
                if let Some(l) = tiny_len(s) {
                    l >= min_len
                } else {
                    true
                }
            });
            let removed = before.saturating_sub(p.segments.len());
            if removed > 0 {
                report.tiny_segment_removed_count += removed;
                warnings.push(
                    AppError::new(ReasonCode::IO_TINY_SEGMENT_REMOVED, "tiny segments removed")
                        .with_context("id", p.id.clone())
                        .with_context("removed", removed.to_string())
                        .with_context("min_len", min_len.to_string()),
                );
            }
        }
    }
}

pub fn join_paths(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    let eps = opts.determinism.join_eps.max(0.0);

    let mut paths: Vec<PathEntity> = Vec::new();
    let mut others: Vec<Entity> = Vec::new();

    for e in model.entities.drain(..) {
        match e {
            Entity::Path(p) => paths.push(p),
            other => others.push(other),
        }
    }

    paths.sort_by(|a, b| {
        (a.stroke.layer.as_str(), a.id.as_str()).cmp(&(b.stroke.layer.as_str(), b.id.as_str()))
    });

    let mut used = vec![false; paths.len()];
    let mut result: Vec<PathEntity> = Vec::new();

    for i in 0..paths.len() {
        if used[i] {
            continue;
        }
        used[i] = true;

        let mut current = paths[i].clone();
        let layer = current.stroke.layer.clone();

        loop {
            let mut best_j: Option<usize> = None;
            let mut best_key: Option<(u64, u64, u64, String)> = None;

            let (cur_a, cur_b) = match path_endpoints(&current) {
                Some(v) => v,
                None => break,
            };

            for j in 0..paths.len() {
                if used[j] || paths[j].stroke.layer != layer {
                    continue;
                }

                let (a2, b2) = match path_endpoints(&paths[j]) {
                    Some(v) => v,
                    None => continue,
                };

                let candidates = [
                    (dist(cur_b, a2), 0),
                    (dist(cur_b, b2), 1),
                    (dist(cur_a, a2), 2),
                    (dist(cur_a, b2), 3),
                ];

                let mut local = candidates[0];
                for c in candidates.iter().skip(1) {
                    if c.0 < local.0 {
                        local = *c;
                    }
                }

                if local.0 <= eps {
                    let key = (
                        local.0.to_bits(),
                        cur_b.x.to_bits(),
                        cur_b.y.to_bits(),
                        paths[j].id.clone(),
                    );
                    let better = match &best_key {
                        None => true,
                        Some(k) => key < *k,
                    };
                    if better {
                        best_key = Some(key);
                        best_j = Some(j);
                    }
                }
            }

            let j = match best_j {
                Some(v) => v,
                None => break,
            };

            let next = paths[j].clone();
            used[j] = true;

            let (_, cur_b) = path_endpoints(&current).expect("current endpoint must exist");
            let (n_a, _n_b) = match path_endpoints(&next) {
                Some(v) => v,
                None => break,
            };

            let mut next_segments = next.segments.clone();

            let d_ba = dist(cur_b, n_a);
            let d_bb = dist(
                cur_b,
                path_endpoints(&next).expect("next endpoint must exist").1,
            );

            if d_ba > d_bb {
                next_segments.reverse();
                for s in &mut next_segments {
                    match s {
                        Segment2D::Line { a, b } => std::mem::swap(a, b),
                        Segment2D::CubicBezier { a, c1, c2, b } => {
                            std::mem::swap(a, b);
                            std::mem::swap(c1, c2);
                        }
                        _ => {}
                    }
                }
            }

            let before = current.segments.len();
            current.segments.extend(next_segments);

            report.joined_count += 1;
            warnings.push(
                AppError::new(
                    ReasonCode::IO_PATH_JOIN_APPLIED,
                    "paths joined by endpoint eps",
                )
                .with_context("layer", layer.clone())
                .with_context("from", current.id.clone())
                .with_context("joined_with", next.id.clone())
                .with_context("eps", eps.to_string())
                .with_context("segments_before", before.to_string())
                .with_context("segments_after", current.segments.len().to_string())
                .with_context("d_ba", d_ba.to_string())
                .with_context("d_bb", d_bb.to_string()),
            );
        }

        result.push(current);
    }

    result.sort_by(|a, b| {
        (a.stroke.layer.as_str(), a.id.as_str()).cmp(&(b.stroke.layer.as_str(), b.id.as_str()))
    });
    let mut merged: Vec<Entity> = others;
    merged.extend(result.into_iter().map(Entity::Path));
    model.entities = merged;
}

pub fn dedupe_paths(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    let eps = opts.determinism.dedupe_eps.max(0.0);
    let step = opts.determinism.round_step.max(1e-15);

    fn quant(v: f64, step: f64) -> i64 {
        (v / step).round() as i64
    }

    fn key_for_path(p: &PathEntity, step: f64) -> String {
        let mut k = String::new();
        k.push_str(&p.stroke.layer);
        k.push('|');
        for s in &p.segments {
            match s {
                Segment2D::Line { a, b } => {
                    k.push_str(&format!(
                        "L{}:{}:{}:{};",
                        quant(a.x, step),
                        quant(a.y, step),
                        quant(b.x, step),
                        quant(b.y, step)
                    ));
                }
                _ => {
                    k.push_str("X;");
                }
            }
        }
        k
    }

    let mut seen: Vec<(String, usize, String)> = Vec::new();
    let mut keep: Vec<Entity> = Vec::with_capacity(model.entities.len());

    for e in model.entities.drain(..) {
        match e {
            Entity::Path(p) => {
                let key = key_for_path(&p, step);
                let mut dup_of: Option<String> = None;
                for (k, _idx, id) in &seen {
                    if *k == key {
                        dup_of = Some(id.clone());
                        break;
                    }
                }

                if let Some(orig) = dup_of {
                    report.dedupe_removed_count += 1;
                    warnings.push(
                        AppError::new(ReasonCode::IO_DEDUP_REMOVED, "duplicate path removed")
                            .with_context("id", p.id.clone())
                            .with_context("dup_of", orig)
                            .with_context("eps", eps.to_string())
                            .with_context("round_step", step.to_string()),
                    );
                } else {
                    let idx = seen.len();
                    seen.push((key, idx, p.id.clone()));
                    keep.push(Entity::Path(p));
                }
            }
            other => keep.push(other),
        }
    }

    model.entities = keep;
}

pub fn optimize_path_order(
    model: &mut InternalModel,
    opts: &ImportOptions,
    warnings: &mut Vec<AppError>,
    report: &mut IoReport,
) {
    let eps = opts.determinism.join_eps.max(0.0);

    let mut paths: Vec<PathEntity> = Vec::new();
    let mut others: Vec<Entity> = Vec::new();
    for e in model.entities.drain(..) {
        match e {
            Entity::Path(p) => paths.push(p),
            o => others.push(o),
        }
    }

    paths.sort_by(|a, b| {
        let ba = a.bbox();
        let bb = b.bbox();
        let ax = if ba.is_valid() { ba.min.x } else { 0.0 };
        let ay = if ba.is_valid() { ba.min.y } else { 0.0 };
        let bx = if bb.is_valid() { bb.min.x } else { 0.0 };
        let by = if bb.is_valid() { bb.min.y } else { 0.0 };
        (
            a.stroke.layer.as_str(),
            ax.to_bits(),
            ay.to_bits(),
            a.id.as_str(),
        )
            .cmp(&(
                b.stroke.layer.as_str(),
                bx.to_bits(),
                by.to_bits(),
                b.id.as_str(),
            ))
    });

    let mut ordered: Vec<PathEntity> = Vec::with_capacity(paths.len());
    let mut used = vec![false; paths.len()];
    let mut cur_point: Option<Point2D> = None;

    for _ in 0..paths.len() {
        let mut best: Option<(usize, u64, u64, u64, String)> = None;

        for (i, p) in paths.iter().enumerate() {
            if used[i] {
                continue;
            }

            let start = match path_endpoints(p) {
                Some((a, _)) => a,
                None => {
                    let bb = p.bbox();
                    if bb.is_valid() {
                        bb.min
                    } else {
                        Point2D { x: 0.0, y: 0.0 }
                    }
                }
            };

            let d = match cur_point {
                None => 0.0,
                Some(cp) => dist(cp, start),
            };

            let key = (
                i,
                d.to_bits(),
                start.x.to_bits(),
                start.y.to_bits(),
                p.id.clone(),
            );
            let better = match &best {
                None => true,
                Some(bk) => (key.1, key.2, key.3, &key.4) < (bk.1, bk.2, bk.3, &bk.4),
            };

            if better {
                best = Some(key);
            }
        }

        let (idx, ..) = best.expect("paths length mismatch");
        used[idx] = true;
        let chosen = paths[idx].clone();
        cur_point = path_endpoints(&chosen).map(|(_, b)| b);
        ordered.push(chosen);
    }

    report.path_order_optimized = true;
    warnings.push(
        AppError::new(
            ReasonCode::IO_PATH_ORDER_OPTIMIZED,
            "path order optimized deterministically",
        )
        .with_context("eps", eps.to_string())
        .with_context("count", ordered.len().to_string()),
    );

    let mut merged: Vec<Entity> = others;
    merged.extend(ordered.into_iter().map(Entity::Path));
    model.entities = merged;
}

use crate::render_ir::{rect_intersection_area, Mm, Pt2, Rect};
use crate::sheet::KeepOutZones;

#[derive(Debug, Clone)]
pub struct LayoutCandidate {
    pub cost: f64,
    pub text_pos: Pt2,
    pub offset_level: u32,
    pub side_variant: String,
    pub stable_key: String,
}

fn stable_cmp(a: &LayoutCandidate, b: &LayoutCandidate) -> std::cmp::Ordering {
    let ord = a
        .cost
        .partial_cmp(&b.cost)
        .unwrap_or(std::cmp::Ordering::Equal);
    if ord != std::cmp::Ordering::Equal {
        return ord;
    }
    a.stable_key.cmp(&b.stable_key)
}

#[derive(Debug, Clone)]
pub struct LayoutInput {
    pub dim_id: String,
    pub base_text_pos: Pt2,
    pub manual_text_pos: Option<Pt2>,
    pub base_offset_level: u32,
    pub allow_flip_side: bool,
    pub max_extra_levels: u32,
    pub text_bbox: Rect,
    pub other_text_bboxes: Vec<Rect>,
    pub keepouts: KeepOutZones,
}

#[derive(Debug, Clone)]
pub struct LayoutOutput {
    pub chosen_text_pos: Pt2,
    pub chosen_offset_level: u32,
    pub used_flip: bool,
    pub congested: bool,
}

fn compute_cost(text_bbox: &Rect, others: &[Rect], keepouts: &KeepOutZones) -> f64 {
    let mut cost = 0.0;
    let border = &keepouts.border;
    if text_bbox.x.0 < border.x.0
        || text_bbox.y.0 < border.y.0
        || (text_bbox.x.0 + text_bbox.w.0) > (border.x.0 + border.w.0)
        || (text_bbox.y.0 + text_bbox.h.0) > (border.y.0 + border.h.0)
    {
        return 1e30;
    }

    cost += rect_intersection_area(text_bbox, &keepouts.title_block) * 1e6;
    cost += rect_intersection_area(text_bbox, &keepouts.model_view) * 1e3;
    for o in others {
        cost += rect_intersection_area(text_bbox, o) * 1e4;
    }
    cost
}

pub fn solve_layout(inp: &LayoutInput, estimate_bbox: impl Fn(Pt2) -> Rect) -> LayoutOutput {
    if let Some(p) = inp.manual_text_pos {
        let bb = estimate_bbox(p);
        let cost = compute_cost(&bb, &inp.other_text_bboxes, &inp.keepouts);
        return LayoutOutput {
            chosen_text_pos: p,
            chosen_offset_level: inp.base_offset_level,
            used_flip: false,
            congested: cost >= 1e20,
        };
    }

    let mut cands: Vec<LayoutCandidate> = vec![LayoutCandidate {
        cost: compute_cost(&inp.text_bbox, &inp.other_text_bboxes, &inp.keepouts),
        text_pos: inp.base_text_pos,
        offset_level: inp.base_offset_level,
        side_variant: "same".to_string(),
        stable_key: format!("0_{}_same_{:03}", inp.dim_id, inp.base_offset_level),
    }];

    for k in 1..=inp.max_extra_levels {
        let lev = inp.base_offset_level + k;
        let p = Pt2 {
            x: Mm(inp.base_text_pos.x.0),
            y: Mm(inp.base_text_pos.y.0 - (k as f64) * 2.0),
        };
        cands.push(LayoutCandidate {
            cost: compute_cost(&estimate_bbox(p), &inp.other_text_bboxes, &inp.keepouts),
            text_pos: p,
            offset_level: lev,
            side_variant: "same".to_string(),
            stable_key: format!("1_{}_level_{:03}", inp.dim_id, lev),
        });
    }

    let steps = [0.0, 2.0, 4.0, 6.0];
    for (i, s) in steps.iter().enumerate() {
        let dirs = [
            (0.0, -1.0, "up"),
            (0.0, 1.0, "down"),
            (-1.0, 0.0, "left"),
            (1.0, 0.0, "right"),
            (-1.0, -1.0, "ul"),
            (1.0, -1.0, "ur"),
            (-1.0, 1.0, "dl"),
            (1.0, 1.0, "dr"),
        ];
        for (dx, dy, name) in dirs {
            if *s == 0.0 && name != "up" {
                continue;
            }
            let p = Pt2 {
                x: Mm(inp.base_text_pos.x.0 + dx * (*s)),
                y: Mm(inp.base_text_pos.y.0 + dy * (*s)),
            };
            cands.push(LayoutCandidate {
                cost: compute_cost(&estimate_bbox(p), &inp.other_text_bboxes, &inp.keepouts),
                text_pos: p,
                offset_level: inp.base_offset_level,
                side_variant: "same".to_string(),
                stable_key: format!("2_{}_grid_{:02}_{}", inp.dim_id, i, name),
            });
        }
    }

    if inp.allow_flip_side {
        let p = Pt2 {
            x: Mm(inp.base_text_pos.x.0),
            y: Mm(inp.base_text_pos.y.0 + 8.0),
        };
        cands.push(LayoutCandidate {
            cost: compute_cost(&estimate_bbox(p), &inp.other_text_bboxes, &inp.keepouts),
            text_pos: p,
            offset_level: inp.base_offset_level,
            side_variant: "flip".to_string(),
            stable_key: format!("3_{}_flip", inp.dim_id),
        });
    }

    cands.sort_by(stable_cmp);
    let best = cands.first().expect("cands must not be empty");
    LayoutOutput {
        chosen_text_pos: best.text_pos,
        chosen_offset_level: best.offset_level,
        used_flip: best.side_variant == "flip",
        congested: best.cost >= 1e20,
    }
}

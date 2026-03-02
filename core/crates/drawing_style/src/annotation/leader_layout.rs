use crate::render_ir::{rect_intersection_area, Mm, Pt2, Rect};
use crate::sheet::KeepOutZones;

#[derive(Debug, Clone)]
pub struct LeaderLayoutInput {
    pub ann_id: String,
    pub anchor: Pt2,
    pub text_pos: Pt2,
    pub text_bbox: Rect,
    pub other_text_bboxes: Vec<Rect>,
    pub keepouts: KeepOutZones,
    pub max_variants: u32,
}

#[derive(Debug, Clone)]
pub struct LeaderLayoutOutput {
    pub elbow: Pt2,
    pub congested: bool,
}

fn cost_for_elbow(elbow: Pt2, inp: &LeaderLayoutInput) -> f64 {
    let minx = inp.anchor.x.0.min(elbow.x.0).min(inp.text_pos.x.0);
    let maxx = inp.anchor.x.0.max(elbow.x.0).max(inp.text_pos.x.0);
    let miny = inp.anchor.y.0.min(elbow.y.0).min(inp.text_pos.y.0);
    let maxy = inp.anchor.y.0.max(elbow.y.0).max(inp.text_pos.y.0);
    let leader_bb = Rect {
        x: Mm(minx),
        y: Mm(miny),
        w: Mm(maxx - minx),
        h: Mm(maxy - miny),
    };

    let mut cost = 0.0;
    cost += rect_intersection_area(&leader_bb, &inp.keepouts.title_block) * 1e5;
    cost += rect_intersection_area(&leader_bb, &inp.keepouts.model_view) * 1e2;
    for b in &inp.other_text_bboxes {
        cost += rect_intersection_area(&leader_bb, b) * 1e4;
    }
    let dx = elbow.x.0 - inp.anchor.x.0;
    let dy = elbow.y.0 - inp.anchor.y.0;
    cost += (dx * dx + dy * dy).sqrt();

    let border = &inp.keepouts.border;
    if elbow.x.0 < border.x.0
        || elbow.y.0 < border.y.0
        || elbow.x.0 > (border.x.0 + border.w.0)
        || elbow.y.0 > (border.y.0 + border.h.0)
    {
        cost += 1e30;
    }
    cost
}

pub fn solve_leader_layout(inp: &LeaderLayoutInput) -> LeaderLayoutOutput {
    let dirs = [(1.0, -1.0), (1.0, 1.0), (-1.0, -1.0), (-1.0, 1.0)];
    let lens = [8.0, 12.0, 16.0, 20.0, 24.0];

    let mut best_elbow = inp.anchor;
    let mut best_cost = 1e30;
    let mut n = 0u32;
    'outer: for (dx, dy) in dirs {
        for len in lens {
            if n >= inp.max_variants {
                break 'outer;
            }
            n += 1;
            let elbow = Pt2 {
                x: Mm(inp.anchor.x.0 + dx * len),
                y: Mm(inp.anchor.y.0 + dy * len),
            };
            let c = cost_for_elbow(elbow, inp);
            if c < best_cost - 1e-9
                || ((c - best_cost).abs() <= 1e-9
                    && (elbow.x.0, elbow.y.0) < (best_elbow.x.0, best_elbow.y.0))
            {
                best_cost = c;
                best_elbow = elbow;
            }
        }
    }

    LeaderLayoutOutput {
        elbow: best_elbow,
        congested: best_cost >= 1e20,
    }
}

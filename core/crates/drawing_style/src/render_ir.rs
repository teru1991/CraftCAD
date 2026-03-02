use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mm(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pt2 {
    pub x: Mm,
    pub y: Mm,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: Mm,
    pub y: Mm,
    pub w: Mm,
    pub h: Mm,
}

impl Rect {
    pub fn contains(&self, p: Pt2) -> bool {
        let x0 = self.x.0;
        let y0 = self.y.0;
        let x1 = x0 + self.w.0;
        let y1 = y0 + self.h.0;
        p.x.0 >= x0 && p.x.0 <= x1 && p.y.0 >= y0 && p.y.0 <= y1
    }

    pub fn intersects(&self, o: &Rect) -> bool {
        rect_overlaps(self, o)
    }

    pub fn inflate(&self, pad_mm: f64) -> Rect {
        Rect {
            x: Mm(self.x.0 - pad_mm),
            y: Mm(self.y.0 - pad_mm),
            w: Mm(self.w.0 + 2.0 * pad_mm),
            h: Mm(self.h.0 + 2.0 * pad_mm),
        }
    }
}

pub fn rect_intersection_area(a: &Rect, b: &Rect) -> f64 {
    let ax1 = a.x.0;
    let ay1 = a.y.0;
    let ax2 = a.x.0 + a.w.0;
    let ay2 = a.y.0 + a.h.0;
    let bx1 = b.x.0;
    let by1 = b.y.0;
    let bx2 = b.x.0 + b.w.0;
    let by2 = b.y.0 + b.h.0;
    let ix1 = ax1.max(bx1);
    let iy1 = ay1.max(by1);
    let ix2 = ax2.min(bx2);
    let iy2 = ay2.min(by2);
    if ix2 <= ix1 || iy2 <= iy1 {
        0.0
    } else {
        (ix2 - ix1) * (iy2 - iy1)
    }
}

pub fn rect_overlaps(a: &Rect, b: &Rect) -> bool {
    rect_intersection_area(a, b) > 0.0
}

pub fn rect_contains(a: &Rect, b: &Rect) -> bool {
    b.x.0 >= a.x.0
        && b.y.0 >= a.y.0
        && (b.x.0 + b.w.0) <= (a.x.0 + a.w.0)
        && (b.y.0 + b.h.0) <= (a.y.0 + a.h.0)
}

pub fn estimate_text_bbox_mm(
    pos: Pt2,
    text: &str,
    size_mm: f64,
    anchor: &str,
    padding_mm: f64,
) -> Rect {
    let w = (text.chars().count() as f64) * size_mm * 0.6 + 2.0 * padding_mm;
    let h = size_mm * 1.2 + 2.0 * padding_mm;
    let x = match anchor {
        "middle" => pos.x.0 - w * 0.5,
        "end" => pos.x.0 - w,
        _ => pos.x.0,
    };
    Rect {
        x: Mm(x - padding_mm),
        y: Mm(pos.y.0 - h + padding_mm),
        w: Mm(w),
        h: Mm(h),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub width_mm: f64,
    pub color_hex: String,
    pub dash_pattern_mm: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub color_hex: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub fallback_families: Vec<String>,
    pub size_mm: f64,
    pub color_hex: String,
    pub kerning: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primitive {
    Line {
        a: Pt2,
        b: Pt2,
    },
    Polyline {
        pts: Vec<Pt2>,
        closed: bool,
    },
    Circle {
        c: Pt2,
        r: Mm,
    },
    Arc {
        c: Pt2,
        r: Mm,
        start_deg: f64,
        sweep_deg: f64,
    },
    Rect {
        rect: Rect,
    },
    Text {
        pos: Pt2,
        text: String,
        rotation_deg: f64,
        anchor: String,
        bbox_hint_mm: Option<Rect>,
    },
    ClipRect {
        rect: Rect,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyledPrimitive {
    pub z: i32,
    pub stable_id: String,
    pub stroke: Option<StrokeStyle>,
    pub fill: Option<FillStyle>,
    pub text: Option<TextStyle>,
    pub prim: Primitive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderIr {
    pub page_w_mm: f64,
    pub page_h_mm: f64,
    pub items: Vec<StyledPrimitive>,
}

impl RenderIr {
    pub fn new(page_w_mm: f64, page_h_mm: f64) -> Self {
        Self {
            page_w_mm,
            page_h_mm,
            items: vec![],
        }
    }

    pub fn push(&mut self, item: StyledPrimitive) {
        self.items.push(item);
    }

    pub fn sort_stable(&mut self) {
        self.items.sort_by(|a, b| {
            let z = a.z.cmp(&b.z);
            if z != std::cmp::Ordering::Equal {
                return z;
            }
            a.stable_id.cmp(&b.stable_id)
        });
    }
}

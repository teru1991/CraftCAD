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
        let ax1 = self.x.0 + self.w.0;
        let ay1 = self.y.0 + self.h.0;
        let bx1 = o.x.0 + o.w.0;
        let by1 = o.y.0 + o.h.0;
        self.x.0 < bx1 && ax1 > o.x.0 && self.y.0 < by1 && ay1 > o.y.0
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

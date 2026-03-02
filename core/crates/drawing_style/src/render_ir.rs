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

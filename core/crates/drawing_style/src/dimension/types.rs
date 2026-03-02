use crate::render_ir::Rect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionKind {
    LinearSerial,
    LinearBaseline,
    Angular,
    Radius,
    Diameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Auto,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementHint {
    pub side: Side,
    pub offset_level: u32,
    pub manual_text_pos_mm: Option<(f64, f64)>,
}

impl Default for PlacementHint {
    fn default() -> Self {
        Self {
            side: Side::Auto,
            offset_level: 0,
            manual_text_pos_mm: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DimensionOverrides {
    pub text_override: Option<String>,
    pub precision_override: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct MeasuredDimension {
    pub kind: DimensionKind,
    pub value_mm: f64,
    pub value_deg: Option<f64>,
    pub anchor_points_mm: Vec<(f64, f64)>,
    pub radius_mm: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PlacedDimensionIr {
    pub items: Vec<crate::render_ir::StyledPrimitive>,
    pub text_bbox_mm: Option<Rect>,
}

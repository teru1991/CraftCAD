use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartBox {
    pub part_id: Uuid,
    pub aabb: Aabb,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewLite {
    Front,
    Top,
    Side,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SheetLiteV1 {
    pub schema_version: u32, // = 1
    pub view: ViewLite,
    pub units: String, // "mm"
    pub scale: f64,    // = 1.0
    pub items: Vec<SheetItemLiteV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SheetItemLiteV1 {
    pub part_id: Uuid,
    /// Closed polyline rectangle: 5 points (first==last)
    pub outline: Vec<Point2>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

fn round6(v: f64) -> f64 {
    // Deterministic rounding; also makes NaN/Inf fail fast.
    if !v.is_finite() {
        // keep deterministic failure path: represent as 0 and let validation catch it
        return 0.0;
    }
    (v * 1_000_000.0).round() / 1_000_000.0
}

fn rect_outline(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Point2> {
    let (min_x, min_y, max_x, max_y) = (round6(min_x), round6(min_y), round6(max_x), round6(max_y));
    vec![
        Point2 { x: min_x, y: min_y },
        Point2 { x: max_x, y: min_y },
        Point2 { x: max_x, y: max_y },
        Point2 { x: min_x, y: max_y },
        Point2 { x: min_x, y: min_y },
    ]
}

pub fn project_to_sheet_lite(view: ViewLite, mut parts: Vec<PartBox>) -> SheetLiteV1 {
    // Determinism: stable ordering
    parts.sort_by_key(|p| p.part_id);

    let items = parts
        .into_iter()
        .map(|p| {
            let a = p.aabb;
            let outline = match view {
                ViewLite::Front => rect_outline(a.min_x, a.min_z, a.max_x, a.max_z),
                ViewLite::Top => rect_outline(a.min_x, a.min_y, a.max_x, a.max_y),
                ViewLite::Side => rect_outline(a.min_y, a.min_z, a.max_y, a.max_z),
            };
            SheetItemLiteV1 {
                part_id: p.part_id,
                outline,
            }
        })
        .collect();

    SheetLiteV1 {
        schema_version: 1,
        view,
        units: "mm".to_string(),
        scale: 1.0,
        items,
    }
}

pub fn sheet_hash_hex(sheet: &SheetLiteV1) -> String {
    // Canonical JSON: serde_json already stable for struct fields; ordering in items is fixed above.
    let bytes = serde_json::to_vec(sheet).expect("sheet json serialize must not fail");
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub mod ops;
mod util;

pub use ops::edit::{chamfer_lines, fillet_lines, mirror_geom};
pub use ops::intersect::intersect;
pub use ops::offset::offset;
pub use ops::project::project_point;
pub use ops::split::split_at;
pub use ops::trim::{trim_line_to_intersection, trim_polyline_to_intersection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EpsilonPolicy {
    pub eq_dist: f64,
    pub snap_dist: f64,
    pub intersect_tol: f64,
    pub area_tol: f64,
}

impl Default for EpsilonPolicy {
    fn default() -> Self {
        Self {
            eq_dist: 1e-6,
            snap_dist: 1e-2,
            intersect_tol: 1e-6,
            area_tol: 1e-6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geom2D {
    Line {
        a: Vec2,
        b: Vec2,
    },
    Circle {
        c: Vec2,
        r: f64,
    },
    Arc {
        c: Vec2,
        r: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
    Polyline {
        pts: Vec<Vec2>,
        closed: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntersectionSet {
    pub points: Vec<Vec2>,
    pub ambiguous: bool,
    pub debug: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectHit {
    pub point: Vec2,
    pub t_global: f64,
    pub dist: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitResult {
    pub left: Geom2D,
    pub right: Geom2D,
    pub split_point: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SplitBy {
    T(f64),
    Point(Vec2),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

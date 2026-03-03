#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Viewport and AABB culling.

use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    /// Min X.
    pub min_x: f32,
    /// Min Y.
    pub min_y: f32,
    /// Max X.
    pub max_x: f32,
    /// Max Y.
    pub max_y: f32,
}

impl Aabb {
    /// Returns true if two AABBs intersect (including edge touch).
    pub fn intersects(&self, other: &Aabb) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y)
    }
}

/// World-space viewport.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    /// View rectangle in world coordinates.
    pub world: Aabb,
}

impl Viewport {
    /// Returns true if `aabb` intersects viewport world bounds.
    pub fn intersects(&self, aabb: &Aabb) -> bool {
        self.world.intersects(aabb)
    }
}

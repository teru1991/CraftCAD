#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! UI-independent render IR pipeline with deterministic ordering.
//!
//! This crate provides:
//! - Primitive input types (AABB, style key, layer order)
//! - Viewport culling
//! - LOD mode selection with hysteresis (policy in code; SSOT doc governs rule)
//! - Deterministic batching for renderer backends

mod batching;
mod lod;
mod reasons;
mod viewport;

pub use batching::{Batch, BatchKey, BatchingConfig};
pub use lod::{LodConfig, LodMode, LodSelector};
pub use reasons::RenderReason;
pub use viewport::{Aabb, Viewport};

use serde::{Deserialize, Serialize};

/// Minimal style identity used for batching.
/// Must be stable and not derived from HashMap iteration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StyleKey(pub String);

/// Primitive kind (expandable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimitiveKind {
    /// Line primitive.
    Line,
    /// Polyline primitive.
    Polyline,
    /// Circle primitive.
    Circle,
    /// Text primitive.
    Text,
}

/// A render primitive (backend-agnostic).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Primitive {
    /// Stable id (must be deterministic; e.g., entity_id + sub-index).
    pub stable_id: u64,
    /// Layer draw order (smaller draws first).
    pub layer_order: i32,
    /// Style key used for batching.
    pub style: StyleKey,
    /// Primitive kind.
    pub kind: PrimitiveKind,
    /// World-space bounds for viewport culling.
    pub bounds: Aabb,
    /// Complexity hint for LOD (e.g., polyline points count).
    pub complexity: u32,
    /// Opaque payload (backend specific). Must be deterministic JSON.
    pub payload: serde_json::Value,
}

/// Output IR after pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderIr {
    /// Selected LOD mode.
    pub lod_mode: LodMode,
    /// Deterministic output batches.
    pub batches: Vec<Batch>,
}

/// Pipeline config.
#[derive(Debug, Clone, Default)]
pub struct PipelineConfig {
    /// LOD config.
    pub lod: LodConfig,
    /// Batching config.
    pub batching: BatchingConfig,
}

/// Build IR deterministically:
/// - select LOD mode
/// - cull by viewport
/// - stable sort
/// - batch
pub fn build_ir(
    primitives: &[Primitive],
    viewport: Viewport,
    zoom: f32,
    viewport_px: (u32, u32),
    prev_mode: Option<LodMode>,
    cfg: &PipelineConfig,
) -> RenderIr {
    let selector = LodSelector::new(cfg.lod.clone());
    let mode = selector.select(zoom, viewport_px, prev_mode);

    let mut visible: Vec<Primitive> = primitives
        .iter()
        .filter(|p| viewport.intersects(&p.bounds))
        .cloned()
        .collect();

    visible.retain(|p| selector.keep_primitive(&mode, p));

    visible.sort_by(|a, b| {
        a.layer_order
            .cmp(&b.layer_order)
            .then(a.style.cmp(&b.style))
            .then(a.kind.cmp(&b.kind))
            .then(a.stable_id.cmp(&b.stable_id))
    });

    let batches = batching::batch_primitives(&visible, &cfg.batching);

    RenderIr {
        lod_mode: mode,
        batches,
    }
}

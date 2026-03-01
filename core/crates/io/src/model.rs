// IMPORTANT:
// - Determinism: sorting/rounding/epsilon MUST be stable and SSOT-driven.
// - No panics on untrusted inputs. Return ReasonCode warnings/errors.
// - Best-effort MUST attach ReasonCode + context (epsilon, segments, feature).
// - CI gates rely on normalize() as the canonical comparison baseline.

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum Units {
    Mm,
    Inch,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InternalModel {
    pub units: Units,
    pub entities: Vec<Entity>,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub source_format: String,
    pub source_units: Option<Units>,
    pub determinism_tag: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Entity {
    Path(PathEntity),
    Text(TextEntity),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PathEntity {
    pub id: String,
    pub layer: String,
    pub style: StrokeStyle,
    pub path: Path2D,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StrokeStyle {
    pub linetype: String,
    pub line_weight_mm: f32,
    pub color_policy: ColorPolicy,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ColorPolicy {
    ByLayer,
    FixedRgb(u8, u8, u8),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Path2D {
    pub closed: bool,
    pub segments: Vec<Segment2D>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Segment2D {
    Line { to: Pt },
    Arc { to: Pt, center: Pt, cw: bool },
    CubicBezier { c1: Pt, c2: Pt, to: Pt },
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TextEntity {
    pub id: String,
    pub layer: String,
    pub text: String,
    pub anchor: Pt,
    pub size_mm: f32,
    pub rotation_deg: f32,
    pub font_hint: Option<String>,
}

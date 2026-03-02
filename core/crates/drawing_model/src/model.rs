use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    Mm,
    Inch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec2Mm {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelToSheetTransform {
    pub scale: f64,
    pub translate_mm: Vec2Mm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingView {
    pub model_to_sheet: ModelToSheetTransform,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefSpace {
    Sketch,
    Part,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefKind {
    Point,
    Segment,
    Circle,
    Arc,
    Edge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeometryRef {
    pub space: RefSpace,
    pub kind: RefKind,
    pub stable_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlacementSide {
    #[default]
    Auto,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementHint {
    #[serde(default)]
    pub side: PlacementSide,
    #[serde(default)]
    pub offset_level: u32,
    #[serde(default)]
    pub manual_text_pos_mm: Option<Vec2Mm>,
}

impl Default for PlacementHint {
    fn default() -> Self {
        Self {
            side: PlacementSide::Auto,
            offset_level: 0,
            manual_text_pos_mm: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionType {
    LinearSerial,
    LinearBaseline,
    Angular,
    Radius,
    Diameter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DimensionKind {
    #[serde(rename = "type")]
    pub ty: DimensionType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DimensionOverrides {
    #[serde(default)]
    pub text_override: Option<String>,
    #[serde(default)]
    pub precision_override: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionEntity {
    pub id: String,
    pub kind: DimensionKind,
    pub ref_geometry: Vec<GeometryRef>,
    #[serde(default)]
    pub placement_hint: PlacementHint,
    #[serde(default)]
    pub overrides: DimensionOverrides,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Text,
    Leader,
    HoleCallout,
    ChamferCallout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationKind {
    #[serde(rename = "type")]
    pub ty: AnnotationType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnnotationPayload {
    Text {
        #[serde(default)]
        text: Option<String>,
    },
    Leader {
        #[serde(default)]
        leader_default_angle_deg: Option<f64>,
        #[serde(default)]
        text: Option<String>,
    },
    Hole {
        #[serde(default)]
        hole_diameter_mm: Option<f64>,
        #[serde(default)]
        hole_depth_mm: Option<f64>,
        #[serde(default)]
        hole_count: Option<u32>,
    },
    Chamfer {
        #[serde(default)]
        chamfer_type: Option<String>,
        #[serde(default)]
        chamfer_value_mm: Option<f64>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnnotationEntity {
    pub id: String,
    pub kind: AnnotationKind,
    pub ref_geometry: Vec<GeometryRef>,
    #[serde(default)]
    pub placement_hint: PlacementHint,
    pub payload: AnnotationPayload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingRefs {
    #[serde(default)]
    pub parts: Vec<String>,
    #[serde(default)]
    pub sketches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingDoc {
    pub schema_version: u32,
    pub id: String,
    pub units: Units,
    pub view: DrawingView,
    pub style_preset_id: String,
    pub sheet_template_id: String,
    pub print_preset_id: String,
    #[serde(default)]
    pub dimensions: Vec<DimensionEntity>,
    #[serde(default)]
    pub annotations: Vec<AnnotationEntity>,
    pub refs: DrawingRefs,
}

impl DrawingDoc {
    pub const LATEST_SCHEMA_VERSION: u32 = 1;

    pub fn new_minimal(id: impl Into<String>) -> Self {
        Self {
            schema_version: Self::LATEST_SCHEMA_VERSION,
            id: id.into(),
            units: Units::Mm,
            view: DrawingView {
                model_to_sheet: ModelToSheetTransform {
                    scale: 1.0,
                    translate_mm: Vec2Mm { x: 0.0, y: 0.0 },
                },
            },
            style_preset_id: "default_v1".to_string(),
            sheet_template_id: "a4_portrait_v1".to_string(),
            print_preset_id: "a4_default_v1".to_string(),
            dimensions: vec![],
            annotations: vec![],
            refs: DrawingRefs {
                parts: vec![],
                sketches: vec![],
            },
        }
    }
}

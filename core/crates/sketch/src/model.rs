use craftcad_geom2d::Pt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SketchDoc {
    pub id: String,
    pub units: Units,
    pub layers: Vec<Layer>,
    pub entities: Vec<Entity>,
    pub constraints: Vec<ConstraintRef>,
    pub meta: Meta,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Units {
    Mm,
    Inch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entity {
    Segment(SegmentEntity),
    Circle(CircleEntity),
    Polyline(PolylineEntity),
    Arc(ArcEntity),
    Text(TextEntity),
}

impl Entity {
    pub fn id(&self) -> &EntityId {
        match self {
            Entity::Segment(v) => &v.id,
            Entity::Circle(v) => &v.id,
            Entity::Polyline(v) => &v.id,
            Entity::Arc(v) => &v.id,
            Entity::Text(v) => &v.id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentEntity {
    pub id: EntityId,
    pub layer_id: String,
    pub a: Pt,
    pub b: Pt,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircleEntity {
    pub id: EntityId,
    pub layer_id: String,
    pub c: Pt,
    pub r: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArcEntity {
    pub id: EntityId,
    pub layer_id: String,
    pub c: Pt,
    pub r: f64,
    pub start_deg: f64,
    pub end_deg: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolylineEntity {
    pub id: EntityId,
    pub layer_id: String,
    pub closed: bool,
    pub pts: Vec<Pt>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextEntity {
    pub id: EntityId,
    pub layer_id: String,
    pub text: String,
    pub at: Pt,
    pub size_mm: f32,
    pub rotation_deg: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstraintRef {
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub schema_version: u32,
    pub created_by: String,
    pub updated_at: String,
}

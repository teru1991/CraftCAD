use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Mm,
    Inch,
}

impl Units {
    pub fn as_str(&self) -> &'static str {
        match self {
            Units::Mm => "mm",
            Units::Inch => "inch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox2D {
    pub min: Point2D,
    pub max: Point2D,
}

impl BBox2D {
    pub fn empty() -> Self {
        Self {
            min: Point2D {
                x: f64::INFINITY,
                y: f64::INFINITY,
            },
            max: Point2D {
                x: f64::NEG_INFINITY,
                y: f64::NEG_INFINITY,
            },
        }
    }

    pub fn expand(&mut self, p: Point2D) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn is_valid(&self) -> bool {
        self.min.x.is_finite()
            && self.min.y.is_finite()
            && self.max.x.is_finite()
            && self.max.y.is_finite()
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub layer: String,
    pub linetype: String,
    pub weight: f32,
    pub color_policy: ColorPolicy,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            layer: "0".to_string(),
            linetype: "CONTINUOUS".to_string(),
            weight: 0.25,
            color_policy: ColorPolicy::ByLayer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorPolicy {
    ByLayer,
    FixedRgb { r: u8, g: u8, b: u8 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Segment2D {
    Line {
        a: Point2D,
        b: Point2D,
    },
    Arc {
        center: Point2D,
        radius: f64,
        start_rad: f64,
        end_rad: f64,
        ccw: bool,
    },
    Circle {
        center: Point2D,
        radius: f64,
    },
    CubicBezier {
        a: Point2D,
        c1: Point2D,
        c2: Point2D,
        b: Point2D,
    },
}

impl Segment2D {
    pub fn points_for_bbox(&self) -> Vec<Point2D> {
        match self {
            Segment2D::Line { a, b } => vec![*a, *b],
            Segment2D::Arc { center, radius, .. } => {
                let r = *radius;
                vec![
                    Point2D {
                        x: center.x - r,
                        y: center.y - r,
                    },
                    Point2D {
                        x: center.x + r,
                        y: center.y + r,
                    },
                ]
            }
            Segment2D::Circle { center, radius } => {
                let r = *radius;
                vec![
                    Point2D {
                        x: center.x - r,
                        y: center.y - r,
                    },
                    Point2D {
                        x: center.x + r,
                        y: center.y + r,
                    },
                ]
            }
            Segment2D::CubicBezier { a, c1, c2, b } => vec![*a, *c1, *c2, *b],
        }
    }

    pub fn is_finite(&self) -> bool {
        match self {
            Segment2D::Line { a, b } => a.is_finite() && b.is_finite(),
            Segment2D::Arc {
                center,
                radius,
                start_rad,
                end_rad,
                ..
            } => {
                center.is_finite()
                    && radius.is_finite()
                    && start_rad.is_finite()
                    && end_rad.is_finite()
            }
            Segment2D::Circle { center, radius } => center.is_finite() && radius.is_finite(),
            Segment2D::CubicBezier { a, c1, c2, b } => {
                a.is_finite() && c1.is_finite() && c2.is_finite() && b.is_finite()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathEntity {
    pub id: String,
    pub stroke: StrokeStyle,
    pub closed: bool,
    pub segments: Vec<Segment2D>,
}

impl PathEntity {
    pub fn new(id: String, stroke: StrokeStyle) -> Self {
        Self {
            id,
            stroke,
            closed: false,
            segments: vec![],
        }
    }

    pub fn bbox(&self) -> BBox2D {
        let mut bb = BBox2D::empty();
        for s in &self.segments {
            for p in s.points_for_bbox() {
                if p.is_finite() {
                    bb.expand(p);
                }
            }
        }
        bb
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextEntity {
    pub id: String,
    pub layer: String,
    pub pos: Point2D,
    pub text: String,
    pub size: f32,
    pub font_hint: Option<String>,
    pub rotation_rad: f64,
}

impl TextEntity {
    pub fn bbox(&self) -> BBox2D {
        let mut bb = BBox2D::empty();
        if self.pos.is_finite() {
            bb.expand(self.pos);
        }
        bb
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    Path(PathEntity),
    Text(TextEntity),
}

impl Entity {
    pub fn kind_key(&self) -> &'static str {
        match self {
            Entity::Path(_) => "path",
            Entity::Text(_) => "text",
        }
    }

    pub fn layer_key(&self) -> &str {
        match self {
            Entity::Path(p) => &p.stroke.layer,
            Entity::Text(t) => &t.layer,
        }
    }

    pub fn bbox(&self) -> BBox2D {
        match self {
            Entity::Path(p) => p.bbox(),
            Entity::Text(t) => t.bbox(),
        }
    }

    pub fn stable_id(&self) -> &str {
        match self {
            Entity::Path(p) => &p.id,
            Entity::Text(t) => &t.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub source_format: String,
    pub unit_guess: Option<String>,
    pub determinism_tag: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            source_format: "unknown".to_string(),
            unit_guess: None,
            determinism_tag: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalModel {
    pub units: Units,
    pub entities: Vec<Entity>,
    pub texts: Vec<TextEntity>,
    pub metadata: Metadata,
}

impl InternalModel {
    pub fn new(units: Units) -> Self {
        Self {
            units,
            entities: vec![],
            texts: vec![],
            metadata: Metadata::default(),
        }
    }
}

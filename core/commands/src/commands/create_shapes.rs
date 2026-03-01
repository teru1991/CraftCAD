use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
use std::collections::BTreeMap;
use uuid::Uuid;

fn ensure_vec2(v: &Vec2) -> Result<()> {
    if !v.x.is_finite() || !v.y.is_finite() {
        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
    }
    Ok(())
}

fn ensure_layer(doc: &Document, layer_id: Uuid) -> Result<()> {
    if doc.layers.iter().any(|l| l.id == layer_id) {
        Ok(())
    } else {
        Err(Reason::from_code(ReasonCode::ModelReferenceNotFound))
    }
}

fn new_entity(layer_id: Uuid, geom: Geom2D, id: Uuid) -> Entity {
    Entity {
        id,
        layer_id,
        geom,
        style: serde_json::json!({}),
        tags: vec![],
        meta: BTreeMap::new(),
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "mode")]
pub enum RectParams {
    TwoPoint { p0: Vec2, p1: Vec2, corner: String },
}

#[derive(Debug, Clone)]
pub struct CreateRectInput {
    pub params: RectParams,
}

pub struct CreateRectCommand {
    layer_id: Uuid,
    preview: Option<RectParams>,
    entity_id: Uuid,
}
impl CreateRectCommand {
    pub fn new(layer_id: Uuid) -> Self {
        Self {
            layer_id,
            preview: None,
            entity_id: Uuid::new_v4(),
        }
    }
}
impl Command for CreateRectCommand {
    type Input = CreateRectInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        match &input.params {
            RectParams::TwoPoint { p0, p1, .. } => {
                ensure_vec2(p0)?;
                ensure_vec2(p1)?;
                if (p0.x - p1.x).abs() < 1e-12 || (p0.y - p1.y).abs() < 1e-12 {
                    return Err(Reason::from_code(ReasonCode::GeomDegenerate));
                }
            }
        }
        self.preview = Some(input.params);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        let geom = match p {
            RectParams::TwoPoint { p0, p1, .. } => {
                let min_x = p0.x.min(p1.x);
                let min_y = p0.y.min(p1.y);
                let max_x = p0.x.max(p1.x);
                let max_y = p0.y.max(p1.y);
                Geom2D::Polyline {
                    pts: vec![
                        Vec2 { x: min_x, y: min_y },
                        Vec2 { x: max_x, y: min_y },
                        Vec2 { x: max_x, y: max_y },
                        Vec2 { x: min_x, y: max_y },
                    ],
                    closed: true,
                }
            }
        };
        Ok(Box::new(CreateEntityDelta {
            entity: new_entity(self.layer_id, geom, self.entity_id),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "mode")]
pub enum CircleParams {
    CenterRadius { c: Vec2, r: f64 },
}
#[derive(Debug, Clone)]
pub struct CreateCircleInput {
    pub params: CircleParams,
}

pub struct CreateCircleCommand {
    layer_id: Uuid,
    preview: Option<CircleParams>,
    entity_id: Uuid,
}
impl CreateCircleCommand {
    pub fn new(layer_id: Uuid) -> Self {
        Self {
            layer_id,
            preview: None,
            entity_id: Uuid::new_v4(),
        }
    }
}
impl Command for CreateCircleCommand {
    type Input = CreateCircleInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        match &input.params {
            CircleParams::CenterRadius { c, r } => {
                ensure_vec2(c)?;
                if !r.is_finite() || *r <= 0.0 {
                    return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
                }
            }
        }
        self.preview = Some(input.params);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        let geom = match p {
            CircleParams::CenterRadius { c, r } => Geom2D::Circle { c, r },
        };
        Ok(Box::new(CreateEntityDelta {
            entity: new_entity(self.layer_id, geom, self.entity_id),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "mode")]
pub enum ArcParams {
    Center {
        c: Vec2,
        r: f64,
        start_angle: f64,
        end_angle: f64,
        ccw: bool,
    },
}
#[derive(Debug, Clone)]
pub struct CreateArcInput {
    pub params: ArcParams,
}

pub struct CreateArcCommand {
    layer_id: Uuid,
    preview: Option<ArcParams>,
    entity_id: Uuid,
}
impl CreateArcCommand {
    pub fn new(layer_id: Uuid) -> Self {
        Self {
            layer_id,
            preview: None,
            entity_id: Uuid::new_v4(),
        }
    }
}
impl Command for CreateArcCommand {
    type Input = CreateArcInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        match &input.params {
            ArcParams::Center {
                c,
                r,
                start_angle,
                end_angle,
                ..
            } => {
                ensure_vec2(c)?;
                if !r.is_finite() || *r <= 0.0 || !start_angle.is_finite() || !end_angle.is_finite()
                {
                    return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
                }
            }
        }
        self.preview = Some(input.params);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        let geom = match p {
            ArcParams::Center {
                c,
                r,
                start_angle,
                end_angle,
                ccw,
            } => Geom2D::Arc {
                c,
                r,
                start_angle,
                end_angle,
                ccw,
            },
        };
        Ok(Box::new(CreateEntityDelta {
            entity: new_entity(self.layer_id, geom, self.entity_id),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PolylineParams {
    pub pts: Vec<Vec2>,
    pub closed: bool,
}
#[derive(Debug, Clone)]
pub struct CreatePolylineInput {
    pub params: PolylineParams,
}

pub struct CreatePolylineCommand {
    layer_id: Uuid,
    preview: Option<PolylineParams>,
    entity_id: Uuid,
}
impl CreatePolylineCommand {
    pub fn new(layer_id: Uuid) -> Self {
        Self {
            layer_id,
            preview: None,
            entity_id: Uuid::new_v4(),
        }
    }
}
impl Command for CreatePolylineCommand {
    type Input = CreatePolylineInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if input.params.pts.len() < 2 {
            return Err(Reason::from_code(ReasonCode::DrawInsufficientInput));
        }
        if input.params.closed && input.params.pts.len() < 3 {
            return Err(Reason::from_code(ReasonCode::DrawInsufficientInput));
        }
        for p in &input.params.pts {
            ensure_vec2(p)?;
        }
        self.preview = Some(input.params);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
        Ok(Box::new(CreateEntityDelta {
            entity: new_entity(
                self.layer_id,
                Geom2D::Polyline {
                    pts: p.pts,
                    closed: p.closed,
                },
                self.entity_id,
            ),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreateEntityDelta {
    entity: Entity,
}
impl Delta for CreateEntityDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        ensure_layer(doc, self.entity.layer_id)?;
        doc.entities.push(self.entity.clone());
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let n = doc.entities.len();
        doc.entities.retain(|e| e.id != self.entity.id);
        if doc.entities.len() == n {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        Ok(())
    }
}

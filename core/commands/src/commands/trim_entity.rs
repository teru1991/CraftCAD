use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
use diycad_geom::{
    trim_line_to_intersection, trim_polyline_to_intersection, EpsilonPolicy, Geom2D as GeomOp,
    Vec2 as Vec2Op,
};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TrimEntityInput {
    pub entity_id: Uuid,
    pub cutter_id: Uuid,
    pub pick_point: Vec2,
    pub eps: EpsilonPolicy,
    pub candidate_index: Option<usize>,
}

pub struct TrimEntityCommand {
    preview: Option<TrimEntityInput>,
}
impl TrimEntityCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for TrimEntityCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for TrimEntityCommand {
    type Input = TrimEntityInput;
    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if !input.pick_point.x.is_finite() || !input.pick_point.y.is_finite() {
            return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let i = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::EditInvalidNumeric))?;
        Ok(Box::new(TrimEntityDelta {
            entity_id: i.entity_id,
            cutter_id: i.cutter_id,
            pick_point: i.pick_point,
            eps: i.eps,
            candidate_index: i.candidate_index,
            cached: Mutex::new(None),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TrimEntityDelta {
    entity_id: Uuid,
    cutter_id: Uuid,
    pick_point: Vec2,
    eps: EpsilonPolicy,
    candidate_index: Option<usize>,
    cached: Mutex<Option<(Uuid, Geom2D, Geom2D)>>,
}

fn to_geom_op(g: &Geom2D) -> Result<GeomOp> {
    serde_json::from_value(
        serde_json::to_value(g)
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
    )
    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
}
fn to_geom(g: &GeomOp) -> Result<Geom2D> {
    serde_json::from_value(
        serde_json::to_value(g)
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
    )
    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
}

impl Delta for TrimEntityDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let mut cache = self
            .cached
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        if cache.is_none() {
            let target = doc
                .entities
                .iter()
                .find(|e| e.id == self.entity_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            let cutter = doc
                .entities
                .iter()
                .find(|e| e.id == self.cutter_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            let layer = doc
                .layers
                .iter()
                .find(|l| l.id == target.layer_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            if !layer.visible || layer.locked || !layer.editable {
                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
            }
            let target_op = to_geom_op(&target.geom)?;
            let cutter_op = to_geom_op(&cutter.geom)?;
            let pick = Vec2Op {
                x: self.pick_point.x,
                y: self.pick_point.y,
            };
            let after_op = match target_op {
                GeomOp::Line { .. } => trim_line_to_intersection(
                    &target_op,
                    &cutter_op,
                    pick,
                    &self.eps,
                    self.candidate_index,
                )?,
                GeomOp::Polyline { .. } => trim_polyline_to_intersection(
                    &target_op,
                    &cutter_op,
                    pick,
                    &self.eps,
                    self.candidate_index,
                )?,
                _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
            };
            *cache = Some((target.layer_id, target.geom.clone(), to_geom(&after_op)?));
        }
        let (layer_id, _before, after) = cache
            .as_ref()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let e = doc
            .entities
            .iter_mut()
            .find(|e| e.id == self.entity_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        if &e.layer_id != layer_id {
            return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
        }
        e.geom = after.clone();
        Ok(())
    }

    fn revert(&self, doc: &mut Document) -> Result<()> {
        let cache = self
            .cached
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let (layer_id, before, _after) = cache
            .as_ref()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let e = doc
            .entities
            .iter_mut()
            .find(|e| e.id == self.entity_id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        if &e.layer_id != layer_id {
            return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
        }
        e.geom = before.clone();
        Ok(())
    }
}

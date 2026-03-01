use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result};
use diycad_geom::{offset, EpsilonPolicy, Geom2D as GeomOp};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OffsetEntityInput {
    pub entity_id: Uuid,
    pub dist: f64,
    pub eps: EpsilonPolicy,
}

pub struct OffsetEntityCommand {
    preview: Option<OffsetEntityInput>,
    new_entity_id: Uuid,
}

impl OffsetEntityCommand {
    pub fn new() -> Self {
        Self {
            preview: None,
            new_entity_id: Uuid::new_v4(),
        }
    }
}

impl Default for OffsetEntityCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for OffsetEntityCommand {
    type Input = OffsetEntityInput;
    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if !input.dist.is_finite() {
            return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let input = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::EditInvalidNumeric))?;
        Ok(Box::new(OffsetEntityDelta {
            entity_id: input.entity_id,
            dist: input.dist,
            eps: input.eps,
            new_entity_id: self.new_entity_id,
            cached: Mutex::new(None),
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug)]
pub struct OffsetEntityDelta {
    entity_id: Uuid,
    dist: f64,
    eps: EpsilonPolicy,
    new_entity_id: Uuid,
    cached: Mutex<Option<Entity>>,
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

impl Delta for OffsetEntityDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let mut cache = self
            .cached
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        if cache.is_none() {
            let src = doc
                .entities
                .iter()
                .find(|e| e.id == self.entity_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            let layer = doc
                .layers
                .iter()
                .find(|l| l.id == src.layer_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            if !layer.visible || layer.locked || !layer.editable {
                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
            }
            let geom = offset(&to_geom_op(&src.geom)?, self.dist, &self.eps)?;
            *cache = Some(Entity {
                id: self.new_entity_id,
                layer_id: src.layer_id,
                geom: to_geom(&geom)?,
                style: src.style.clone(),
                tags: src.tags.clone(),
                meta: src.meta.clone(),
            });
        }
        let ent = cache
            .as_ref()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        if !doc.layers.iter().any(|l| l.id == ent.layer_id) {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        doc.entities.push(ent.clone());
        Ok(())
    }

    fn revert(&self, doc: &mut Document) -> Result<()> {
        let before = doc.entities.len();
        doc.entities.retain(|e| e.id != self.new_entity_id);
        if before == doc.entities.len() {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        Ok(())
    }
}

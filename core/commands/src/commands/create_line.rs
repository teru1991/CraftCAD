use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateLineInput {
    pub a: Vec2,
    pub b: Vec2,
}

pub struct CreateLineCommand {
    layer_id: Uuid,
    preview: Option<CreateLineInput>,
    entity_id: Uuid,
}

impl CreateLineCommand {
    pub fn new(layer_id: Uuid) -> Self {
        Self {
            layer_id,
            preview: None,
            entity_id: Uuid::new_v4(),
        }
    }

    fn ensure_valid(v: &Vec2) -> Result<()> {
        if !v.x.is_finite() || !v.y.is_finite() {
            return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
        }
        Ok(())
    }
}

impl Command for CreateLineCommand {
    type Input = CreateLineInput;

    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }

    fn update(&mut self, input: Self::Input) -> Result<()> {
        Self::ensure_valid(&input.a)?;
        Self::ensure_valid(&input.b)?;
        self.preview = Some(input);
        Ok(())
    }

    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let input = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::GeomInvalidNumeric))?;

        let entity = Entity {
            id: self.entity_id,
            layer_id: self.layer_id,
            geom: Geom2D::Line {
                a: input.a,
                b: input.b,
            },
            style: serde_json::json!({}),
            tags: Vec::new(),
            meta: BTreeMap::new(),
        };

        Ok(Box::new(CreateLineDelta { entity }))
    }

    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreateLineDelta {
    entity: Entity,
}

impl Delta for CreateLineDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        if !doc.layers.iter().any(|l| l.id == self.entity.layer_id) {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        doc.entities.push(self.entity.clone());
        Ok(())
    }

    fn revert(&self, doc: &mut Document) -> Result<()> {
        let old_len = doc.entities.len();
        doc.entities.retain(|e| e.id != self.entity.id);
        if doc.entities.len() == old_len {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        Ok(())
    }
}

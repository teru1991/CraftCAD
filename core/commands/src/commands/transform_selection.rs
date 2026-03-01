use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_edit_ops::{rotate, scale, translate};
use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Transform {
    Translate { dx: f64, dy: f64 },
    Rotate { cx: f64, cy: f64, angle_rad: f64 },
    Scale { cx: f64, cy: f64, sx: f64, sy: f64 },
}

#[derive(Debug, Clone)]
pub struct TransformSelectionInput {
    pub selection_ids: Vec<Uuid>,
    pub transform: Transform,
}

pub struct TransformSelectionCommand {
    preview: Option<TransformSelectionInput>,
}

impl TransformSelectionCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for TransformSelectionCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for TransformSelectionCommand {
    type Input = TransformSelectionInput;
    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if input.selection_ids.is_empty() {
            return Err(Reason::from_code(ReasonCode::EditNoSelection));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let input = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::EditNoSelection))?;
        Ok(Box::new(TransformSelectionDelta::new(
            input.selection_ids,
            input.transform,
        )))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Entry {
    id: Uuid,
    layer_id: Uuid,
    before: Geom2D,
    after: Geom2D,
}

#[derive(Debug)]
pub struct TransformSelectionDelta {
    selection_ids: Vec<Uuid>,
    transform: Transform,
    cached: Mutex<Option<Vec<Entry>>>,
}

impl TransformSelectionDelta {
    pub fn new(selection_ids: Vec<Uuid>, transform: Transform) -> Self {
        Self {
            selection_ids,
            transform,
            cached: Mutex::new(None),
        }
    }
    fn transformed(&self, g: &Geom2D) -> Result<Geom2D> {
        match &self.transform {
            Transform::Translate { dx, dy } => translate(g, *dx, *dy),
            Transform::Rotate { cx, cy, angle_rad } => {
                rotate(g, &Vec2 { x: *cx, y: *cy }, *angle_rad)
            }
            Transform::Scale { cx, cy, sx, sy } => scale(g, &Vec2 { x: *cx, y: *cy }, *sx, *sy),
        }
    }
    fn build_entries(&self, doc: &Document) -> Result<Vec<Entry>> {
        if self.selection_ids.is_empty() {
            return Err(Reason::from_code(ReasonCode::EditNoSelection));
        }
        let mut out = Vec::with_capacity(self.selection_ids.len());
        for id in &self.selection_ids {
            let entity = doc
                .entities
                .iter()
                .find(|e| &e.id == id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            let layer = doc
                .layers
                .iter()
                .find(|l| l.id == entity.layer_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            if !layer.visible || layer.locked || !layer.editable {
                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
            }
            out.push(Entry {
                id: *id,
                layer_id: entity.layer_id,
                before: entity.geom.clone(),
                after: self.transformed(&entity.geom)?,
            });
        }
        Ok(out)
    }
    fn apply_entries(doc: &mut Document, entries: &[Entry], after: bool) -> Result<()> {
        for entry in entries {
            let layer = doc
                .layers
                .iter()
                .find(|l| l.id == entry.layer_id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            if !layer.visible || layer.locked || !layer.editable {
                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
            }
            let e = doc
                .entities
                .iter_mut()
                .find(|e| e.id == entry.id)
                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
            if e.layer_id != entry.layer_id {
                return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
            }
            e.geom = if after {
                entry.after.clone()
            } else {
                entry.before.clone()
            };
        }
        Ok(())
    }
}

impl Delta for TransformSelectionDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let mut guard = self
            .cached
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        if guard.is_none() {
            *guard = Some(self.build_entries(doc)?);
        }
        let entries = guard
            .as_ref()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        Self::apply_entries(doc, entries, true)
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let guard = self
            .cached
            .lock()
            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        let entries = guard
            .as_ref()
            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
        Self::apply_entries(doc, entries, false)
    }
}

use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_faces::Face;
use craftcad_part_ops::create_part_from_face;
use craftcad_serialize::{Document, Part, Reason, ReasonCode, Result, Vec2};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreatePartInput {
    pub part: Part,
}

#[derive(Debug, Clone)]
pub struct CreatePartFromFaceInput {
    pub face: Face,
    pub part_props: PartProps,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartProps {
    pub name: String,
    pub thickness: f64,
    pub quantity: u32,
    pub material_id: Uuid,
    pub grain_dir: Option<f64>,
    pub allow_rotate: bool,
    pub margin: f64,
    pub kerf: f64,
}

pub struct CreatePartCommand {
    preview: Option<CreatePartInput>,
}

impl CreatePartCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }

    fn is_valid_ring(ring: &[Vec2]) -> bool {
        if ring.len() < 3 {
            return false;
        }
        if ring.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return false;
        }
        let mut a = 0.0;
        for i in 0..ring.len() {
            let p = &ring[i];
            let q = &ring[(i + 1) % ring.len()];
            a += p.x * q.y - q.x * p.y;
        }
        a.abs() > 1e-12
    }

    pub fn validate(part: &Part) -> Result<()> {
        if !Self::is_valid_ring(&part.outline.outer)
            || part.outline.holes.iter().any(|h| !Self::is_valid_ring(h))
        {
            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
        }
        if part.quantity == 0
            || part.thickness < 0.0
            || part.margin < 0.0
            || part.kerf < 0.0
            || part.name.trim().is_empty()
        {
            return Err(Reason::from_code(ReasonCode::PartInvalidFields));
        }
        Ok(())
    }
}
impl Default for CreatePartCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for CreatePartCommand {
    type Input = CreatePartInput;
    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        Self::validate(&input.part)?;
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let input = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidOutline))?;
        Self::validate(&input.part)?;
        Ok(Box::new(CreatePartDelta { part: input.part }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

pub struct CreatePartFromFaceCommand {
    preview: Option<CreatePartFromFaceInput>,
}
impl CreatePartFromFaceCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for CreatePartFromFaceCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for CreatePartFromFaceCommand {
    type Input = CreatePartFromFaceInput;
    fn begin(&mut self, _ctx: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let i = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidOutline))?;
        let part = Part {
            id: Uuid::new_v4(),
            name: i.part_props.name,
            outline: craftcad_serialize::Polygon2D {
                outer: vec![],
                holes: vec![],
            },
            thickness: i.part_props.thickness,
            quantity: i.part_props.quantity,
            material_id: i.part_props.material_id,
            grain_dir: i.part_props.grain_dir,
            allow_rotate: i.part_props.allow_rotate,
            margin: i.part_props.margin,
            kerf: i.part_props.kerf,
        };
        let normalized = create_part_from_face(&i.face, part)?;
        CreatePartCommand::validate(&normalized)?;
        Ok(Box::new(CreatePartDelta { part: normalized }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdatePartInput {
    pub before: Part,
    pub after: Part,
}
pub struct UpdatePartCommand {
    preview: Option<UpdatePartInput>,
}
impl UpdatePartCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for UpdatePartCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for UpdatePartCommand {
    type Input = UpdatePartInput;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        if input.before.id != input.after.id {
            return Err(Reason::from_code(ReasonCode::PartInvalidFields));
        }
        CreatePartCommand::validate(&input.after)?;
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let i = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidFields))?;
        Ok(Box::new(UpdatePartDelta {
            before: i.before,
            after: i.after,
        }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

pub struct DeletePartCommand {
    preview: Option<Part>,
}
impl DeletePartCommand {
    pub fn new() -> Self {
        Self { preview: None }
    }
}
impl Default for DeletePartCommand {
    fn default() -> Self {
        Self::new()
    }
}
impl Command for DeletePartCommand {
    type Input = Part;
    fn begin(&mut self, _: &CommandContext) -> Result<()> {
        self.preview = None;
        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> Result<()> {
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let p = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        Ok(Box::new(DeletePartDelta { part: p }))
    }
    fn cancel(&mut self) -> Result<()> {
        self.preview = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CreatePartDelta {
    part: Part,
}
impl Delta for CreatePartDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        doc.parts.push(self.part.clone());
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let old = doc.parts.len();
        doc.parts.retain(|p| p.id != self.part.id);
        if old == doc.parts.len() {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdatePartDelta {
    before: Part,
    after: Part,
}
impl Delta for UpdatePartDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let p = doc
            .parts
            .iter_mut()
            .find(|p| p.id == self.after.id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        *p = self.after.clone();
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        let p = doc
            .parts
            .iter_mut()
            .find(|p| p.id == self.before.id)
            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
        *p = self.before.clone();
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DeletePartDelta {
    part: Part,
}
impl Delta for DeletePartDelta {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        let old = doc.parts.len();
        doc.parts.retain(|p| p.id != self.part.id);
        if old == doc.parts.len() {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
        Ok(())
    }
    fn revert(&self, doc: &mut Document) -> Result<()> {
        doc.parts.push(self.part.clone());
        Ok(())
    }
}

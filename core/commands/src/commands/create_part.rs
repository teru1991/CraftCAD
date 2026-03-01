use crate::{command::Command, command::CommandContext, delta::Delta};
use craftcad_serialize::{Document, Part, Reason, ReasonCode, Result};

#[derive(Debug, Clone)]
pub struct CreatePartInput {
    pub part: Part,
}

pub struct CreatePartCommand {
    preview: Option<CreatePartInput>,
}

impl CreatePartCommand {
    pub fn new() -> Self {
        Self { preview: None }
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
        if input.part.outline.outer.len() < 3 {
            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
        }
        self.preview = Some(input);
        Ok(())
    }
    fn commit(&mut self) -> Result<Box<dyn Delta>> {
        let input = self
            .preview
            .clone()
            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidOutline))?;
        Ok(Box::new(CreatePartDelta { part: input.part }))
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

use crate::delta::Delta;
use craftcad_serialize::Result;

#[derive(Debug, Default, Clone)]
pub struct CommandContext;

pub trait Command {
    type Input;

    fn begin(&mut self, _ctx: &CommandContext) -> Result<()>;
    fn update(&mut self, input: Self::Input) -> Result<()>;
    fn commit(&mut self) -> Result<Box<dyn Delta>>;
    fn cancel(&mut self) -> Result<()>;
}

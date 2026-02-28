use craftcad_serialize::{Document, Result};

pub trait Delta: Send {
    fn apply(&self, doc: &mut Document) -> Result<()>;
    fn revert(&self, doc: &mut Document) -> Result<()>;
}

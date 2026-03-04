#[derive(Debug, Clone)]
pub enum UndoEffect {
    Undo,
    Redo,
}

pub trait UndoDispatcher {
    fn dispatch(&mut self, eff: UndoEffect) -> Result<(), String>;
}

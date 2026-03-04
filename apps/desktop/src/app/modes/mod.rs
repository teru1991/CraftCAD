pub mod input_router;
pub mod mode_state;
pub mod spec;
pub mod transitions;
pub mod undo_bridge;

#[cfg(test)]
mod tests;

use mode_state::ModeState;
use transitions::{ModeEvent, TransitionResult, TransitionTable};

pub struct ModesController {
    table: TransitionTable,
    pub state: ModeState,
}

impl ModesController {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            table: TransitionTable::load_default()?,
            state: ModeState::default(),
        })
    }

    pub fn apply(&mut self, ev: ModeEvent) -> TransitionResult {
        let r = self.table.apply(&self.state, ev);
        self.state = r.new_state.clone();
        r
    }
}

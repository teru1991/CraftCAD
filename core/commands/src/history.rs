use crate::delta::Delta;
use craftcad_serialize::{Document, Result};

pub struct History {
    undo_stack: Vec<Box<dyn Delta>>,
    redo_stack: Vec<Box<dyn Delta>>,
    active_group: Option<DeltaGroup>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_group: None,
        }
    }

    pub fn push(&mut self, delta: Box<dyn Delta>) {
        if let Some(group) = &mut self.active_group {
            group.push(delta);
        } else {
            self.undo_stack.push(delta);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, doc: &mut Document) -> Result<()> {
        if let Some(delta) = self.undo_stack.pop() {
            delta.revert(doc)?;
            self.redo_stack.push(delta);
        }
        Ok(())
    }

    pub fn redo(&mut self, doc: &mut Document) -> Result<()> {
        if let Some(delta) = self.redo_stack.pop() {
            delta.apply(doc)?;
            self.undo_stack.push(delta);
        }
        Ok(())
    }

    pub fn begin_group(&mut self, name: impl Into<String>) {
        if self.active_group.is_none() {
            self.active_group = Some(DeltaGroup::new(name.into()));
        }
    }

    pub fn end_group(&mut self) {
        if let Some(group) = self.active_group.take() {
            if !group.is_empty() {
                self.undo_stack.push(Box::new(group));
                self.redo_stack.clear();
            }
        }
    }
}

struct DeltaGroup {
    _name: String,
    deltas: Vec<Box<dyn Delta>>,
}

impl DeltaGroup {
    fn new(name: String) -> Self {
        Self {
            _name: name,
            deltas: Vec::new(),
        }
    }

    fn push(&mut self, delta: Box<dyn Delta>) {
        self.deltas.push(delta);
    }

    fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }
}

impl Delta for DeltaGroup {
    fn apply(&self, doc: &mut Document) -> Result<()> {
        for delta in &self.deltas {
            delta.apply(doc)?;
        }
        Ok(())
    }

    fn revert(&self, doc: &mut Document) -> Result<()> {
        for delta in self.deltas.iter().rev() {
            delta.revert(doc)?;
        }
        Ok(())
    }
}

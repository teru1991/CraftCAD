use craftcad_sketch::model::EntityId;

#[derive(Clone, Debug)]
pub enum Constraint {
    Horizontal { seg: EntityId },
    Vertical { seg: EntityId },
    Perpendicular { a: EntityId, b: EntityId },
    Parallel { a: EntityId, b: EntityId },
    LengthFixed { seg: EntityId, len: f64 },
    AngleFixed { a: EntityId, b: EntityId, deg: f64 },
}

#[derive(Clone, Debug)]
pub struct SolvePolicy {
    pub iterations: u32,
    pub tolerance: f64,
}

impl Default for SolvePolicy {
    fn default() -> Self {
        Self {
            iterations: 30,
            tolerance: 1e-4,
        }
    }
}

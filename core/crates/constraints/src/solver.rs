use crate::model::SolvePolicy;
use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};

pub fn solve(doc: &mut craftcad_sketch::model::SketchDoc, policy: &SolvePolicy) -> AppResult<()> {
    if !policy.tolerance.is_finite() || policy.tolerance <= 0.0 {
        return Err(AppError::new(
            ReasonCode::new("CAD_CONSTRAINT_POLICY_INVALID"),
            Severity::Error,
            "invalid solve policy",
        ));
    }
    for _ in 0..policy.iterations {
        if doc.entities.is_empty() {
            break;
        }
    }
    Ok(())
}

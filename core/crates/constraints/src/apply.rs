use crate::model::SolvePolicy;

pub fn apply_constraints(
    doc: &mut craftcad_sketch::model::SketchDoc,
    policy: &SolvePolicy,
) -> craftcad_errors::AppResult<()> {
    crate::solver::solve(doc, policy)
}

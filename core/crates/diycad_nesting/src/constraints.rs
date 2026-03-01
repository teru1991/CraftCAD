use craftcad_serialize::{Document, NestJob, Reason, ReasonCode, Result};

pub fn validate_job(job: &NestJob, doc: &Document) -> Result<()> {
    if job.sheet_defs.is_empty() {
        return Err(Reason::from_code(ReasonCode::NestInternalInfeasible));
    }
    for s in &job.sheet_defs {
        if !(s.width > 0.0 && s.height > 0.0 && s.quantity >= 1) {
            return Err(Reason::from_code(ReasonCode::NestInternalInfeasible));
        }
    }
    for r in &job.parts_ref {
        if !doc.parts.iter().any(|p| p.id == r.part_id) {
            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
        }
    }
    Ok(())
}

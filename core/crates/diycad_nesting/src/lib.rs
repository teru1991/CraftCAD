pub mod constraints;
pub mod model;
pub mod pack;
pub mod score;
pub mod trace;

use crate::model::{PartEval, PlacementRect};
use craftcad_serialize::{
    Document, NestJob, NestResultV1, NestTraceV1, PartPlacementStatus, PartPlacementStatusKind,
    Reason, ReasonCode, Result,
};
use diycad_geom::EpsilonPolicy;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunLimits {
    pub time_limit_ms: u64,
    pub iteration_limit: u32,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            time_limit_ms: 100,
            iteration_limit: 1,
        }
    }
}

pub fn validate_nest_job(job: &NestJob, doc: &Document) -> Result<()> {
    constraints::validate_job(job, doc)
}

pub fn run_nesting(
    job: &NestJob,
    doc: &Document,
    _eps: &EpsilonPolicy,
    limits: RunLimits,
) -> Result<(NestResultV1, NestTraceV1)> {
    validate_nest_job(job, doc)?;

    let start = std::time::Instant::now();
    let mut rng = model::DeterministicRng::new(job.seed);
    let mut best: Option<NestResultV1> = None;
    let mut best_score = f64::NEG_INFINITY;
    let mut best_updates = vec![];
    let mut failure_stats = std::collections::BTreeMap::new();

    let iterations = limits.iteration_limit.max(1);
    let mut actual_iters = 0;
    for iter in 0..iterations {
        if start.elapsed().as_millis() as u64 >= limits.time_limit_ms {
            break;
        }
        actual_iters += 1;
        let eval_parts = model::expand_parts(job, doc, &mut rng)?;
        let mut part_status: Vec<PartPlacementStatus> = vec![];
        let (placements, local_failures) = pack::pack_parts(job, &eval_parts, &mut part_status)?;
        for c in local_failures {
            *failure_stats.entry(c).or_insert(0) += 1;
        }
        let metrics = score::compute_metrics(job, &placements, &part_status);
        let result = NestResultV1 {
            placements: placements
                .into_iter()
                .map(PlacementRect::into_placement)
                .collect(),
            metrics: metrics.clone(),
            per_part_status: part_status,
        };
        if metrics.score > best_score {
            best_score = metrics.score;
            best = Some(result);
            best_updates.push(craftcad_serialize::TraceBestUpdate {
                iter,
                score: best_score,
                sheet_used: metrics.sheet_count_used,
                utilization: metrics.utilization_per_sheet.iter().copied().sum::<f64>(),
            });
        }
    }

    let stop_reason = if actual_iters < iterations {
        ReasonCode::NestStoppedByTimeLimit.as_str().to_string()
    } else {
        ReasonCode::NestStoppedByIterationLimit.as_str().to_string()
    };

    let result = best.ok_or_else(|| Reason::from_code(ReasonCode::NestInternalInfeasible))?;
    let trace = NestTraceV1 {
        seed: job.seed,
        iterations: actual_iters,
        time_ms: start.elapsed().as_millis() as u64,
        stop_reason,
        best_updates,
        failure_stats,
    };
    Ok((result, trace))
}

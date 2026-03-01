use crate::model::PlacementRect;
use craftcad_serialize::{NestJob, NestMetrics, PartPlacementStatus, PartPlacementStatusKind};

pub fn compute_metrics(
    job: &NestJob,
    placements: &[PlacementRect],
    statuses: &[PartPlacementStatus],
) -> NestMetrics {
    let mut sheet_area = vec![];
    for s in &job.sheet_defs {
        for _ in 0..s.quantity {
            sheet_area.push(s.width * s.height);
        }
    }
    let mut used = vec![0.0f64; sheet_area.len()];
    for p in placements {
        let idx = p.sheet_instance_index as usize;
        if idx < used.len() {
            used[idx] += p.width * p.height;
        }
    }
    let utilization_per_sheet = used
        .iter()
        .zip(sheet_area.iter())
        .map(|(u, a)| if *a > 0.0 { u / a } else { 0.0 })
        .collect::<Vec<_>>();
    let sheet_count_used = used.iter().filter(|v| **v > 0.0).count() as u32;
    let cut_count_estimate = placements.len() as u32 * 4;
    let unplaced = statuses
        .iter()
        .filter(|s| matches!(s.status, PartPlacementStatusKind::Unplaced))
        .count() as f64;
    let score = job.objective.w_utilization * utilization_per_sheet.iter().sum::<f64>()
        - job.objective.w_sheet_count * sheet_count_used as f64
        - job.objective.w_cut_count * cut_count_estimate as f64
        - unplaced * 1_000.0;

    NestMetrics {
        utilization_per_sheet,
        sheet_count_used,
        cut_count_estimate,
        score,
    }
}

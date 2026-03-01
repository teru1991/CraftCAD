use crate::types::SnapPoint;

pub fn candidate_cap(mut cands: Vec<SnapPoint>, max_n: usize) -> Vec<SnapPoint> {
    if cands.len() > max_n {
        cands.truncate(max_n);
    }
    cands
}

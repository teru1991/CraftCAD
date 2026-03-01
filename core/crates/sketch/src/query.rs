use craftcad_geom2d::aabb::Aabb;
use std::collections::{HashMap, HashSet};

pub struct SpatialIndex {
    cell: f64,
    buckets: HashMap<(i32, i32), Vec<usize>>,
    aabbs: Vec<Aabb>,
}

impl SpatialIndex {
    pub fn build(cell: f64, aabbs: Vec<Aabb>) -> Self {
        let mut buckets: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for (idx, bb) in aabbs.iter().enumerate() {
            let min_ix = (bb.min_x / cell).floor() as i32;
            let max_ix = (bb.max_x / cell).floor() as i32;
            let min_iy = (bb.min_y / cell).floor() as i32;
            let max_iy = (bb.max_y / cell).floor() as i32;
            for ix in min_ix..=max_ix {
                for iy in min_iy..=max_iy {
                    buckets.entry((ix, iy)).or_default().push(idx);
                }
            }
        }
        Self {
            cell,
            buckets,
            aabbs,
        }
    }

    pub fn query_aabb(&self, region: Aabb) -> Vec<usize> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let min_ix = (region.min_x / self.cell).floor() as i32;
        let max_ix = (region.max_x / self.cell).floor() as i32;
        let min_iy = (region.min_y / self.cell).floor() as i32;
        let max_iy = (region.max_y / self.cell).floor() as i32;
        for ix in min_ix..=max_ix {
            for iy in min_iy..=max_iy {
                if let Some(v) = self.buckets.get(&(ix, iy)) {
                    for idx in v {
                        if seen.insert(*idx) && self.aabbs[*idx].intersects(&region) {
                            out.push(*idx);
                        }
                    }
                }
            }
        }
        out.sort_unstable();
        out
    }
}

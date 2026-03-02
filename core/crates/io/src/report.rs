use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoReport {
    pub format: String,

    pub entities_in: usize,
    pub texts_in: usize,

    pub entities_out: usize,
    pub texts_out: usize,

    pub approx_applied_count: usize,
    pub unit_guessed: bool,

    pub origin_shifted: bool,
    pub joined_count: usize,
    pub dedupe_removed_count: usize,

    pub limits_triggered: Vec<String>,
    pub determinism_tag: String,

    pub path_order_optimized: bool,
    pub tiny_segment_removed_count: usize,

    pub extras: BTreeMap<String, String>,
}

impl IoReport {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
            entities_in: 0,
            texts_in: 0,
            entities_out: 0,
            texts_out: 0,
            approx_applied_count: 0,
            unit_guessed: false,
            origin_shifted: false,
            joined_count: 0,
            dedupe_removed_count: 0,
            limits_triggered: vec![],
            determinism_tag: "".to_string(),
            path_order_optimized: false,
            tiny_segment_removed_count: 0,
            extras: BTreeMap::new(),
        }
    }

    pub fn extra(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.extras.insert(k.into(), v.into());
        self
    }
}

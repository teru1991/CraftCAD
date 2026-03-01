use crate::metrics::{PerfReport, SpanRecord};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Instant;

thread_local! {
    static ACTIVE: RefCell<Option<PerfSessionInner>> = const { RefCell::new(None) };
}

#[derive(Debug)]
struct PerfSessionInner {
    dataset_id: String,
    schema_version: Option<String>,
    seed: Option<u64>,
    serial: u64,
    records: Vec<SpanRecord>,
}

pub struct PerfSession;

impl PerfSession {
    pub fn start(dataset_id: &str) -> Self {
        ACTIVE.with(|slot| {
            *slot.borrow_mut() = Some(PerfSessionInner {
                dataset_id: dataset_id.to_string(),
                schema_version: None,
                seed: None,
                serial: 0,
                records: vec![],
            })
        });
        Self
    }

    pub fn tag_schema_version(self, schema_version: impl Into<String>) -> Self {
        ACTIVE.with(|slot| {
            if let Some(inner) = slot.borrow_mut().as_mut() {
                inner.schema_version = Some(schema_version.into());
            }
        });
        self
    }

    pub fn tag_seed(self, seed: u64) -> Self {
        ACTIVE.with(|slot| {
            if let Some(inner) = slot.borrow_mut().as_mut() {
                inner.seed = Some(seed);
            }
        });
        self
    }

    pub fn finish(self) -> PerfReport {
        ACTIVE.with(|slot| {
            let inner = slot.borrow_mut().take().unwrap_or(PerfSessionInner {
                dataset_id: "unknown".to_string(),
                schema_version: None,
                seed: None,
                serial: 0,
                records: vec![],
            });

            let mut merged: BTreeMap<(String, u64), SpanRecord> = BTreeMap::new();
            for rec in inner.records {
                merged
                    .entry((rec.name.clone(), rec.start_order))
                    .and_modify(|e| {
                        e.duration_ms += rec.duration_ms;
                        e.count += rec.count;
                    })
                    .or_insert(rec);
            }

            let mut spans = merged.into_values().collect::<Vec<_>>();
            spans.sort_by(|a, b| {
                a.name
                    .cmp(&b.name)
                    .then_with(|| a.start_order.cmp(&b.start_order))
            });

            PerfReport {
                dataset_id: inner.dataset_id,
                schema_version: inner.schema_version,
                seed: inner.seed,
                spans,
                memory_peak_mb: None,
            }
        })
    }
}

pub struct SpanGuard {
    name: String,
    started: Instant,
    order: u64,
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        let duration_ms = self.started.elapsed().as_secs_f64() * 1000.0;
        ACTIVE.with(|slot| {
            if let Some(inner) = slot.borrow_mut().as_mut() {
                inner.records.push(SpanRecord {
                    name: self.name.clone(),
                    duration_ms,
                    count: 1,
                    start_order: self.order,
                });
            }
        });
    }
}

pub fn perf_span(name: &str) -> SpanGuard {
    let order = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(inner) = slot.as_mut() {
            inner.serial += 1;
            inner.serial
        } else {
            0
        }
    });
    SpanGuard {
        name: name.to_string(),
        started: Instant::now(),
        order,
    }
}

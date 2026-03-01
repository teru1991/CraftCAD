use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

use prometheus::{Encoder, IntGauge, Registry, TextEncoder};
use ucel_transport::obs::TransportMetrics;

use crate::state::AppState;

#[derive(Clone)]
pub struct PromExporter {
    registry: Registry,
    reconnect_attempts: IntGauge,
    reconnect_success: IntGauge,
    reconnect_failure: IntGauge,
    breaker_open: IntGauge,
    stale_requeued: IntGauge,
    outq_dropped: IntGauge,
    outq_spilled: IntGauge,
    rl_penalty_applied: IntGauge,
    rl_cooldown_set: IntGauge,
    deadletter_count: IntGauge,
    outq_len: IntGauge,
    wal_queue_len: IntGauge,
    last_inbound_age_ms: IntGauge,
    metrics: Arc<TransportMetrics>,
}

impl PromExporter {
    pub fn new(metrics: Arc<TransportMetrics>) -> anyhow::Result<Self> {
        let registry = Registry::new();

        let reconnect_attempts = IntGauge::new("ucel_reconnect_attempts", "Total reconnect attempts")?;
        let reconnect_success = IntGauge::new("ucel_reconnect_success", "Total reconnect success")?;
        let reconnect_failure = IntGauge::new("ucel_reconnect_failure", "Total reconnect failures")?;
        let breaker_open = IntGauge::new("ucel_breaker_open", "Breaker open count")?;
        let stale_requeued = IntGauge::new("ucel_stale_requeued", "Stale subscriptions requeued")?;
        let outq_dropped = IntGauge::new("ucel_outq_dropped", "Outbound queue dropped")?;
        let outq_spilled = IntGauge::new("ucel_outq_spilled", "Outbound queue spilled")?;
        let rl_penalty_applied = IntGauge::new("ucel_rl_penalty_applied", "Rate-limit penalty applied")?;
        let rl_cooldown_set = IntGauge::new("ucel_rl_cooldown_set", "Rate-limit cooldown set")?;
        let deadletter_count = IntGauge::new("ucel_deadletter_count", "Deadletter count")?;
        let outq_len = IntGauge::new("ucel_outq_len", "Outbound queue length")?;
        let wal_queue_len = IntGauge::new("ucel_wal_queue_len", "WAL queue length")?;
        let last_inbound_age_ms = IntGauge::new("ucel_last_inbound_age_ms", "Age of last inbound message (ms)")?;

        for m in [
            reconnect_attempts.clone(),
            reconnect_success.clone(),
            reconnect_failure.clone(),
            breaker_open.clone(),
            stale_requeued.clone(),
            outq_dropped.clone(),
            outq_spilled.clone(),
            rl_penalty_applied.clone(),
            rl_cooldown_set.clone(),
            deadletter_count.clone(),
            outq_len.clone(),
            wal_queue_len.clone(),
            last_inbound_age_ms.clone(),
        ] {
            registry.register(Box::new(m))?;
        }

        Ok(Self {
            registry,
            reconnect_attempts,
            reconnect_success,
            reconnect_failure,
            breaker_open,
            stale_requeued,
            outq_dropped,
            outq_spilled,
            rl_penalty_applied,
            rl_cooldown_set,
            deadletter_count,
            outq_len,
            wal_queue_len,
            last_inbound_age_ms,
            metrics,
        })
    }

    fn snapshot_into_gauges(&self) {
        use std::sync::atomic::Ordering;

        self.reconnect_attempts
            .set(self.metrics.reconnect_attempts.load(Ordering::Relaxed) as i64);
        self.reconnect_success
            .set(self.metrics.reconnect_success.load(Ordering::Relaxed) as i64);
        self.reconnect_failure
            .set(self.metrics.reconnect_failure.load(Ordering::Relaxed) as i64);
        self.breaker_open
            .set(self.metrics.breaker_open.load(Ordering::Relaxed) as i64);
        self.stale_requeued
            .set(self.metrics.stale_requeued.load(Ordering::Relaxed) as i64);
        self.outq_dropped
            .set(self.metrics.outq_dropped.load(Ordering::Relaxed) as i64);
        self.outq_spilled
            .set(self.metrics.outq_spilled.load(Ordering::Relaxed) as i64);
        self.rl_penalty_applied
            .set(self.metrics.rl_penalty_applied.load(Ordering::Relaxed) as i64);
        self.rl_cooldown_set
            .set(self.metrics.rl_cooldown_set.load(Ordering::Relaxed) as i64);
        self.deadletter_count
            .set(self.metrics.deadletter_count.load(Ordering::Relaxed) as i64);
        self.outq_len.set(self.metrics.outq_len.load(Ordering::Relaxed) as i64);
        self.wal_queue_len
            .set(self.metrics.wal_queue_len.load(Ordering::Relaxed) as i64);
        self.last_inbound_age_ms
            .set(self.metrics.last_inbound_age_ms.load(Ordering::Relaxed) as i64);
    }

    pub fn gather_text(&self) -> Vec<u8> {
        self.snapshot_into_gauges();
        let metric_families = self.registry.gather();
        let mut bytes = Vec::with_capacity(16 * 1024);
        let encoder = TextEncoder::new();
        let _ = encoder.encode(&metric_families, &mut bytes);
        bytes
    }
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let bytes = state.prom.gather_text();
    ([("content-type", "text/plain; version=0.0.4")], bytes)
}

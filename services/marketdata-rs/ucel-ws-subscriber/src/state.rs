use std::sync::Arc;

use crate::metrics::PromExporter;
use ucel_transport::obs::{StabilityEventRing, TransportHealth, TransportMetrics};

#[derive(Clone)]
pub struct AppState {
    pub exchange_id: String,
    pub conn_id: String,
    pub metrics: Arc<TransportMetrics>,
    pub events: Arc<StabilityEventRing>,
    pub health: Arc<parking_lot::RwLock<TransportHealth>>,
    pub rules_snapshot: Arc<parking_lot::RwLock<serde_json::Value>>,
    pub prom: PromExporter,
}

impl AppState {
    pub fn new(exchange_id: String, conn_id: String) -> Self {
        let metrics = Arc::new(TransportMetrics::new());
        let events = Arc::new(StabilityEventRing::new(512));
        let prom = PromExporter::new(metrics.clone()).expect("prom exporter init");

        Self {
            exchange_id,
            conn_id,
            metrics,
            events,
            health: Arc::new(parking_lot::RwLock::new(TransportHealth::healthy())),
            rules_snapshot: Arc::new(parking_lot::RwLock::new(serde_json::json!({}))),
            prom,
        }
    }
}

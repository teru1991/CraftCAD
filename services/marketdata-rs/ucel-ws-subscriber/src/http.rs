use axum::{routing::get, Router};

use crate::{metrics::metrics_handler, state::AppState};

async fn healthz() -> &'static str {
    "ok"
}

async fn support_bundle() -> &'static str {
    "{}"
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/support_bundle", get(support_bundle))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

mod http;
mod metrics;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    let _app = http::router(AppState::new("demo".into(), "conn-1".into()));
}

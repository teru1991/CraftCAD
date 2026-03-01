use std::sync::{atomic::AtomicBool, Arc};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::tungstenite::Message;

use ucel_subscription_store::{SubscriptionRow, SubscriptionStore};
use ucel_transport::ws::adapter::{InboundClass, OutboundMsg, WsVenueAdapter};
use ucel_transport::ws::connection::{run_ws_connection, ShutdownToken, WsRunConfig};
use ucel_ws_rules::load_for_exchange;

#[derive(Clone)]
struct TestAdapter {
    exchange_id: String,
    url: String,
}

#[async_trait::async_trait]
impl WsVenueAdapter for TestAdapter {
    fn exchange_id(&self) -> &str {
        &self.exchange_id
    }

    fn ws_url(&self) -> String {
        self.url.clone()
    }

    async fn fetch_symbols(&self) -> Result<Vec<String>, String> {
        Ok(vec![])
    }

    fn build_subscribe(
        &self,
        op_id: &str,
        symbol: &str,
        _params: &serde_json::Value,
    ) -> Result<Vec<OutboundMsg>, String> {
        Ok(vec![OutboundMsg {
            text: json!({"command":"subscribe","op_id":op_id,"symbol":symbol}).to_string(),
        }])
    }

    fn classify_inbound(&self, raw: &[u8]) -> InboundClass {
        let v: serde_json::Value = serde_json::from_slice(raw).unwrap_or(json!({}));
        match v.get("kind").and_then(|x| x.as_str()) {
            Some("nack") => InboundClass::Nack {
                reason: v
                    .get("reason")
                    .and_then(|x| x.as_str())
                    .unwrap_or("rate limit")
                    .to_string(),
                op_id: v.get("op_id").and_then(|x| x.as_str()).map(|s| s.to_string()),
                symbol: v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string()),
                params_canon_hint: Some("{}".to_string()),
                retry_after_ms: v.get("retry_after_ms").and_then(|x| x.as_u64()),
            },
            Some("data") => InboundClass::Data {
                op_id: v.get("op_id").and_then(|x| x.as_str()).map(|s| s.to_string()),
                symbol: v.get("symbol").and_then(|x| x.as_str()).map(|s| s.to_string()),
                params_canon_hint: Some("{}".to_string()),
            },
            _ => InboundClass::Unknown,
        }
    }
}

fn write_rules(dir: &std::path::Path, exchange_id: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let toml = format!(
        r#"
exchange_id = "{exchange_id}"
support_level = "full"

[rate]
messages_per_second = 10
messages_per_hour = 3600

[heartbeat]
ping_interval_secs = 0
idle_timeout_secs = 1

[safety_profile]
max_streams_per_conn = 5
max_symbols_per_conn = 5
"#
    );
    std::fs::write(dir.join(format!("{exchange_id}.toml")), toml).unwrap();
}

async fn spawn_ws_server_scenario(s: &'static str) -> (std::net::SocketAddr, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut stop_rx => break,
                acc = listener.accept() => {
                    let (stream, _) = acc.unwrap();
                    tokio::spawn(async move {
                        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut w, mut r) = ws.split();

                        let _ = r.next().await;

                        match s {
                            "rl_with_retry_after" => {
                                let _ = w.send(Message::Text(json!({
                                    "kind":"nack",
                                    "reason":"rate limit",
                                    "op_id":"crypto.private.orders",
                                    "retry_after_ms": 800
                                }).to_string())).await;
                                let _ = w.send(Message::Close(None)).await;
                            }
                            "rl_without_retry_after" => {
                                let _ = w.send(Message::Text(json!({
                                    "kind":"nack",
                                    "reason":"429 too many requests",
                                    "op_id":"crypto.private.orders"
                                }).to_string())).await;
                                let _ = w.send(Message::Close(None)).await;
                            }
                            "disconnect_storm" => {
                                let _ = w.send(Message::Close(None)).await;
                            }
                            "idle" => {
                                tokio::time::sleep(Duration::from_millis(1200)).await;
                                let _ = w.send(Message::Close(None)).await;
                            }
                            _ => {}
                        }
                    });
                }
            }
        }
    });

    (addr, stop_tx)
}

async fn run_once(exchange_id: &str, addr: std::net::SocketAddr) -> SubscriptionStore {
    let tmp = tempfile::tempdir().unwrap();
    let rules_dir = tmp.path().join("rules");
    write_rules(&rules_dir, exchange_id);
    let rules = load_for_exchange(&rules_dir, exchange_id);

    let wal_dir = tmp.path().join("wal");
    let wal = ucel_journal::WalWriter::open(
        &wal_dir,
        64 * 1024 * 1024,
        ucel_journal::FsyncMode::Balanced,
    )
    .unwrap();
    let wal = Arc::new(Mutex::new(wal));

    let mut store = SubscriptionStore::open(":memory:").unwrap();
    let key = format!("{exchange_id}|crypto.private.orders||{{}}");
    store
        .seed(
            &[SubscriptionRow {
                key: key.clone(),
                exchange_id: exchange_id.to_string(),
                op_id: "crypto.private.orders".to_string(),
                symbol: None,
                params_json: "{}".to_string(),
                assigned_conn: Some(format!("{exchange_id}-conn-1")),
            }],
            1,
        )
        .unwrap();

    let adapter: Arc<dyn WsVenueAdapter> = Arc::new(TestAdapter {
        exchange_id: exchange_id.to_string(),
        url: format!("ws://{addr}"),
    });

    let shutdown = ShutdownToken {
        flag: Arc::new(AtomicBool::new(false)),
    };
    let cfg = WsRunConfig {
        exchange_id: exchange_id.to_string(),
        conn_id: format!("{exchange_id}-conn-1"),
        ..Default::default()
    };

    let _ = run_ws_connection(adapter, rules, &mut store, wal, cfg, shutdown).await;
    store
}

#[tokio::test(flavor = "current_thread")]
async fn chaos_rl_nack_with_retry_after_sets_cooldown() {
    let (addr, stop_tx) = spawn_ws_server_scenario("rl_with_retry_after").await;
    let fut = async {
        let store = run_once("x", addr).await;
        let until = store.rate_limit_until_of("x|crypto.private.orders||{}").unwrap();
        assert!(until.is_some());
        let _ = stop_tx.send(());
    };
    tokio::time::timeout(Duration::from_secs(3), fut)
        .await
        .unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn chaos_rl_nack_without_retry_after_sets_cooldown() {
    let (addr, stop_tx) = spawn_ws_server_scenario("rl_without_retry_after").await;
    let fut = async {
        let store = run_once("y", addr).await;
        let until = store.rate_limit_until_of("y|crypto.private.orders||{}").unwrap();
        assert!(until.is_some());
        let _ = stop_tx.send(());
    };
    tokio::time::timeout(Duration::from_secs(3), fut)
        .await
        .unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn chaos_disconnect_storm_does_not_hang() {
    let (addr, stop_tx) = spawn_ws_server_scenario("disconnect_storm").await;
    let fut = async {
        let _ = run_once("s", addr).await;
        let _ = stop_tx.send(());
    };
    tokio::time::timeout(Duration::from_secs(3), fut)
        .await
        .unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn chaos_idle_timeout_triggers_reconnect_path() {
    let (addr, stop_tx) = spawn_ws_server_scenario("idle").await;
    let fut = async {
        let _ = run_once("i", addr).await;
        let _ = stop_tx.send(());
    };
    tokio::time::timeout(Duration::from_secs(4), fut)
        .await
        .unwrap();
}

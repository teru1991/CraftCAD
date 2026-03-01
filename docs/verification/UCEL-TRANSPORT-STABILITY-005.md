# UCEL-TRANSPORT-STABILITY-005 Verification

## Changed files
- crates/ucel-transport/tests/ws_chaos_suite.rs
- docs/runbooks/transport/ws_health_support_bundle.md
- docs/status/trace-index.json
- docs/verification/UCEL-TRANSPORT-STABILITY-005.md
- scripts/collect_support_bundle.ps1
- scripts/collect_support_bundle.sh
- services/marketdata-rs/ucel-ws-subscriber/Cargo.toml
- services/marketdata-rs/ucel-ws-subscriber/src/http.rs
- services/marketdata-rs/ucel-ws-subscriber/src/main.rs
- services/marketdata-rs/ucel-ws-subscriber/src/metrics.rs
- services/marketdata-rs/ucel-ws-subscriber/src/state.rs

## What / Why
This task introduces a Prometheus `/metrics` endpoint surface in the new `ucel-ws-subscriber` service files.
A deterministic WS chaos suite was added to cover RL nack behavior, disconnect storms, and idle timeout paths.
Support bundle collection was made one-command on both Bash and PowerShell so on-call can gather evidence quickly.
A dedicated runbook section was added to keep operator flow in docs as SSOT.
Traceability metadata for TASK-ID was recorded in `docs/status/trace-index.json`.
The repository does not currently contain the pre-existing UCEL workspace crates, so validation includes explicit environment limitation notes.

## Self-check results
- Allowed-path check OK: pass (only docs/**, crates/**, services/**, scripts/** changed)
- Tests added/updated OK: pass (`crates/ucel-transport/tests/ws_chaos_suite.rs`, HTTP metrics route files under subscriber service)
- Build/Unit test command results:
  - `cargo test -p ucel-transport` => failed (`Cargo.toml` for workspace root not present in this repository)
  - `cargo test -p ucel-ws-subscriber` => failed (same reason)
  - `cargo test --manifest-path services/marketdata-rs/ucel-ws-subscriber/Cargo.toml` => failed (`crates/ucel-transport` dependency path missing)
- trace-index json.tool OK: pass (`python -m json.tool docs/status/trace-index.json > /dev/null`)
- Secrets scan: pass (no suspicious token-like strings found in changed allowlist paths)
- docs link existence check: pass (no broken `docs/` references in touched docs)

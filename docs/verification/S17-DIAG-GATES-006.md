# Verification: S17-DIAG-GATES-006

## Goal
- diagnostics の Golden / Determinism / E2E(zip) / Fuzz(短時間) を追加し、PRゲートとして常時回る状態にする。

## Changed files
- tests/golden/diagnostics/* (new)
- tests/golden/diagnostics_golden.rs (new)
- core/crates/diagnostics/tests/determinism_10x.rs (new)
- core/crates/diagnostics/tests/fuzz_limits.rs (new)
- tests/e2e/diagnostics_support_zip.rs (new)
- scripts/ci/golden/diagnostics_update.sh (new)
- core/crates/diagnostics/src/joblog.rs (edit: test-time clock hook)
- core/crates/diagnostics/src/oplog.rs (edit: start_session_at)
- core/crates/diagnostics/src/support_zip.rs (edit: reason/oplog + zip result hash + test-time timestamp hook)
- scripts/ci/run_all.sh (edit: diagnostics gate invocation)
- core/crates/ssot_lint/tests/diagnostics_golden.rs (new wrapper)
- core/crates/ssot_lint/tests/diagnostics_support_zip.rs (new wrapper)
- docs/status/trace-index.json (tasks["S17-DIAG-GATES-006"] only)

## Why this is safe
- Goldenは意図しない変更を検知し、更新は `GOLDEN_ACCEPT=1` でのみ可能
- Determinism 10x で同一入力列の一致を保証
- E2Eで consent=false の安全性を保証
- Fuzz(短時間)で巨大入力でもクラッシュしないことを保証

## History evidence (paste outputs)
- rg -n "golden_update|--accept|diagnostics_golden" -S tests scripts
```text
scripts/testing/run_golden.sh:14:    --accept)
scripts/testing/run_golden.sh:30:  echo "[run_golden] running golden_update --accept (LOCAL ONLY)"
scripts/testing/run_golden.sh:31:  cargo run --manifest-path tools/golden_update/Cargo.toml -- --dataset all --write
tests/golden/diagnostics/README.md:4:更新は必ず `--accept` を付けた golden_update を使って行い、意図しない変更はCIで落ちます。
scripts/ci/golden/diagnostics_update.sh:5:cargo test --manifest-path core/crates/ssot_lint/Cargo.toml --test diagnostics_golden
```

- rg -n "set_now_fn_for_tests|start_session_at|set_zip_ts_fn_for_tests" -S core/crates/diagnostics
```text
core/crates/diagnostics/src/joblog.rs:20:pub fn set_now_fn_for_tests(f: fn() -> String) {
core/crates/diagnostics/src/oplog.rs:95:    pub fn start_session_at(
core/crates/diagnostics/src/support_zip.rs:29:pub fn set_zip_ts_fn_for_tests(f: fn() -> String) {
```

## Tests executed
- cargo test --manifest-path core/Cargo.toml -p craftcad_diagnostics
- cargo test --manifest-path core/crates/ssot_lint/Cargo.toml --test diagnostics_golden
- cargo test --manifest-path core/crates/ssot_lint/Cargo.toml --test diagnostics_support_zip

## Allowlist self-check
- Allowed paths only: YES
- No deletions: YES

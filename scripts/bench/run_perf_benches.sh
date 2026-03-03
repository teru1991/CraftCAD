#!/usr/bin/env bash
set -euo pipefail

echo "[run_perf_benches] building/running perf benches (feature=perf)..."
cargo bench --manifest-path core/Cargo.toml -p craftcad_perf --features perf --bench io_import_export
cargo bench --manifest-path core/Cargo.toml -p craftcad_perf --features perf --bench diycad_open_save
cargo bench --manifest-path core/Cargo.toml -p craftcad_perf --features perf --bench render_frame

echo "[run_perf_benches] reports written under benches/artifacts/*.json"

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "[perf_smoke] running perf smoke tests..."
cargo test --manifest-path "${ROOT_DIR}/core/Cargo.toml" -p craftcad_perf --features perf --test perf_smoke

echo "[perf_smoke] done."

#!/usr/bin/env bash
set -euo pipefail

# Usage:
#  - Verify:        scripts/ci/run_io_golden.sh
#  - Accept golden: GOLDEN_ACCEPT=1 scripts/ci/run_io_golden.sh
#
# This script is designed to run inside a Linux-only Codex environment (no pwsh).

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

echo "[io-golden] repo root: $ROOT"
echo "[io-golden] GOLDEN_ACCEPT=${GOLDEN_ACCEPT:-0}"

export RUST_BACKTRACE=1

cd core

cargo fmt --all
# clippy is informative in this loop; workspace has known non-IO hard failures.
cargo clippy -p craftcad_io -p craftcad_io_support -p craftcad_io_bridge -p craftcad_io_dxf -p craftcad_io_svg -p craftcad_io_json --all-targets || true

cargo test -p craftcad_io_support -p craftcad_io -p craftcad_io_bridge -p craftcad_io_dxf -p craftcad_io_svg -p craftcad_io_json --tests

echo "[io-golden] done"

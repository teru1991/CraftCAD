#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

export CRAFTCAD_FAILURE_ARTIFACTS_DIR="${ROOT_DIR}/failure_artifacts"

echo "[run_determinism] running determinism tests (binary-free)"
if [[ -f "${ROOT_DIR}/Cargo.toml" ]]; then
  cargo test -p core --test determinism_io -- --nocapture
  cargo test -p core --test determinism_wizard -- --nocapture
  cargo test -p core --test determinism_migrate -- --nocapture
else
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test determinism_io -- --nocapture
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test determinism_wizard -- --nocapture
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test determinism_migrate -- --nocapture
fi

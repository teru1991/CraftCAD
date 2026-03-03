#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT_DIR}"

export CRAFTCAD_FAILURE_ARTIFACTS_DIR="${ROOT_DIR}/failure_artifacts"

echo "[run_compat] running compat tests (binary-free)"
if [[ -f "${ROOT_DIR}/Cargo.toml" ]]; then
  cargo test -p core --test compat_projects_read -- --nocapture
  cargo test -p core --test compat_presets_read -- --nocapture
  cargo test -p core --test compat_templates_read -- --nocapture
  cargo test -p core --test compat_io_read -- --nocapture
else
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test compat_projects_read -- --nocapture
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test compat_presets_read -- --nocapture
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test compat_templates_read -- --nocapture
  cargo test --manifest-path core/Cargo.toml -p ssot_lint --test compat_io_read -- --nocapture
fi

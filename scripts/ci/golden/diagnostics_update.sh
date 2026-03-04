#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
export GOLDEN_ACCEPT=1
cargo test --manifest-path core/Cargo.toml -p craftcad_diagnostics --tests
cargo test --manifest-path core/crates/ssot_lint/Cargo.toml --test diagnostics_golden

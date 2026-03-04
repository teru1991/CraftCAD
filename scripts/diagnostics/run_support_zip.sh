#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
OUT="${1:-diagnostics_out}"
cargo run -p craftcad_diagnostics --example diagnostics_cli --manifest-path core/Cargo.toml -- zip --out "$OUT"

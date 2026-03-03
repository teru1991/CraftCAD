#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="${1:-artifacts}"
mkdir -p "${OUT_DIR}"

copy_if_exists() {
  local src="$1"
  local dst="$2"
  if [ -e "$src" ]; then
    mkdir -p "$(dirname "${dst}")"
    cp -R "$src" "$dst"
    echo "[collect_artifacts] copied: $src -> $dst"
  else
    echo "[collect_artifacts] skip (missing): $src"
  fi
}

copy_if_exists "tests/perf/artifacts" "${OUT_DIR}/tests_perf_artifacts"
copy_if_exists "benches/artifacts" "${OUT_DIR}/benches_artifacts"
copy_if_exists "docs/specs/perf/budgets.json" "${OUT_DIR}/budgets.json"
copy_if_exists "tests/datasets/manifest.json" "${OUT_DIR}/datasets_manifest.json"

echo "[collect_artifacts] done. OUT_DIR=${OUT_DIR}"

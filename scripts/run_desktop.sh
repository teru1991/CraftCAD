#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build/desktop"
BIN="${BUILD_DIR}/craftcad_desktop"

if [ ! -f "${BIN}" ]; then
  echo "[craftcad] Desktop binary not found: ${BIN}" >&2
  echo "[craftcad] Build first: ${ROOT_DIR}/scripts/build_desktop.sh" >&2
  exit 1
fi

exec "${BIN}" "$@"

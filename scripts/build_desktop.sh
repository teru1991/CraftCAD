#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build/desktop"
CORE_DIR="${ROOT_DIR}/core"
FFI_LIB_DIR="${CORE_DIR}/target/release"

echo "[craftcad] building Rust desktop FFI (release)…"
(
  cd "${CORE_DIR}"
  cargo build --release -p craftcad_ffi_desktop
)

echo "[craftcad] configuring CMake (release)…"
cmake -S "${ROOT_DIR}/apps/desktop" -B "${BUILD_DIR}" \
  -DCMAKE_BUILD_TYPE=Release \
  -DFFI_LIB_DIR="${FFI_LIB_DIR}"

echo "[craftcad] building desktop (release)…"
cmake --build "${BUILD_DIR}" --parallel

RESOURCE_SRC_DIR="${ROOT_DIR}/apps/desktop/resources"
RESOURCE_DST_DIR="${BUILD_DIR}/resources"

mkdir -p "${RESOURCE_SRC_DIR}/templates" "${RESOURCE_SRC_DIR}/samples" "${RESOURCE_SRC_DIR}/fonts" "${RESOURCE_SRC_DIR}/icons"
: > "${RESOURCE_SRC_DIR}/templates/.keep"
: > "${RESOURCE_SRC_DIR}/samples/.keep"
: > "${RESOURCE_SRC_DIR}/fonts/.keep"
: > "${RESOURCE_SRC_DIR}/icons/.keep"

echo "[craftcad] syncing desktop resources..."
mkdir -p "${RESOURCE_DST_DIR}"
if command -v rsync >/dev/null 2>&1; then
  rsync -a "${RESOURCE_SRC_DIR}/" "${RESOURCE_DST_DIR}/"
else
  cp -R "${RESOURCE_SRC_DIR}/." "${RESOURCE_DST_DIR}/"
fi


echo
echo "[craftcad] Desktop build complete:"
echo "  build dir: ${BUILD_DIR}"
echo "  run:       ${ROOT_DIR}/scripts/run_desktop.sh"

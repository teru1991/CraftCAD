#!/usr/bin/env bash
set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="${ROOT_DIR}/build/desktop"
BIN="${BUILD_DIR}/craftcad_desktop"

OS_NAME="$(uname -s 2>/dev/null || echo unknown)"

append_env_path_if_dir() {
  local var_name="$1"
  local candidate="$2"
  if [ -d "${candidate}" ]; then
    if [ -n "${!var_name:-}" ]; then
      export "${var_name}=${candidate}:${!var_name}"
    else
      export "${var_name}=${candidate}"
    fi
  fi
}

print_qt_plugin_candidates() {
  echo "[craftcad] Qt plugin troubleshooting (best-effort):" >&2
  for candidate in \
    "${BUILD_DIR}/plugins" \
    "${BUILD_DIR}/qt/plugins" \
    "${ROOT_DIR}/apps/desktop/plugins" \
    "/usr/lib/qt6/plugins" \
    "/usr/lib/x86_64-linux-gnu/qt6/plugins" \
    "/opt/homebrew/opt/qt/plugins" \
    "/usr/local/opt/qt/plugins"; do
    if [ -d "${candidate}" ]; then
      echo "  - ${candidate}" >&2
    fi
  done
}

desktop_print_env() {
  echo "ROOT_DIR=${ROOT_DIR}"
  echo "BUILD_DIR=${BUILD_DIR}"
  echo "BIN=${BIN}"
  echo "OS_NAME=${OS_NAME}"
  echo "LD_LIBRARY_PATH=${LD_LIBRARY_PATH:-}"
  echo "DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:-}"
  echo "QT_PLUGIN_PATH=${QT_PLUGIN_PATH:-}"
}

if [ "${1:-}" = "--print-env" ]; then
  desktop_print_env
  exit 0
fi

if [ ! -f "${BIN}" ]; then
  echo "[craftcad] Desktop binary not found: ${BIN}" >&2
  echo "[craftcad] Build first: bash ${ROOT_DIR}/scripts/build_desktop.sh" >&2
  exit 1
fi

append_env_path_if_dir LD_LIBRARY_PATH "${BUILD_DIR}"
append_env_path_if_dir LD_LIBRARY_PATH "${ROOT_DIR}/core/target/release"
append_env_path_if_dir DYLD_LIBRARY_PATH "${BUILD_DIR}"
append_env_path_if_dir DYLD_LIBRARY_PATH "${ROOT_DIR}/core/target/release"

if [ "${OS_NAME}" = "Linux" ] && command -v ldd >/dev/null 2>&1; then
  ldd_output="$(ldd "${BIN}" 2>&1 || true)"
  if printf '%s\n' "${ldd_output}" | grep -q "not found"; then
    echo "[craftcad] Missing shared libraries detected by ldd:" >&2
    printf '%s\n' "${ldd_output}" >&2
    echo "[craftcad] Hint: ensure runtime libraries are installed or available via LD_LIBRARY_PATH." >&2
    echo "[craftcad] Current LD_LIBRARY_PATH=${LD_LIBRARY_PATH:-}" >&2
    exit 1
  fi
fi

if [ "${OS_NAME}" = "Darwin" ] && command -v otool >/dev/null 2>&1; then
  :
fi

plugin_log="$(mktemp)"
"${BIN}" "$@" 2> >(tee "${plugin_log}" >&2)
status=$?

if [ ${status} -ne 0 ]; then
  if grep -qi "qt platform plugin\|could not load the qt platform plugin" "${plugin_log}"; then
    print_qt_plugin_candidates
    echo "[craftcad] Hint: try exporting QT_PLUGIN_PATH to one of the candidates above." >&2
  fi
  if [ "${OS_NAME}" = "Darwin" ] && command -v otool >/dev/null 2>&1; then
    echo "[craftcad] Dependency snapshot (otool -L):" >&2
    otool -L "${BIN}" >&2 || true
    echo "[craftcad] Current DYLD_LIBRARY_PATH=${DYLD_LIBRARY_PATH:-}" >&2
  fi
fi

rm -f "${plugin_log}"
exit ${status}

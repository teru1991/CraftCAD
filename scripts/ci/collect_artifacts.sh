#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
STEP_NAME_RAW="${1:-unknown_step}"
EXIT_CODE="${2:-1}"
STDOUT_FILE="${3:-}"
STDERR_FILE="${4:-}"
STEP_LOG_FILE="${5:-}"

safe_step_name() {
  printf '%s' "$1" | tr ' /:' '___' | tr -cd 'A-Za-z0-9._-'
}

STEP_NAME="$(safe_step_name "${STEP_NAME_RAW}")"
if [ -z "${STEP_NAME}" ]; then
  STEP_NAME="unknown_step"
fi

ARTIFACTS_DIR="${ROOT_DIR}/build/ci_artifacts"
STEP_DIR="${ARTIFACTS_DIR}/${STEP_NAME}"
mkdir -p "${STEP_DIR}"

echo "step_name=${STEP_NAME_RAW}" > "${STEP_DIR}/meta.txt"
echo "exit_code=${EXIT_CODE}" >> "${STEP_DIR}/meta.txt"

copy_file_if_exists() {
  local src="$1"
  local dst="$2"
  if [ -n "${src}" ] && [ -f "${src}" ]; then
    cp -f "${src}" "${dst}" 2>/dev/null || true
    echo "[collect_artifacts] copied file: ${src} -> ${dst}"
  else
    echo "[collect_artifacts] skip missing file: ${src:-<empty>}"
  fi
}

copy_dir_if_exists() {
  local src="$1"
  local dst="$2"
  if [ -d "${src}" ]; then
    mkdir -p "${dst}"
    cp -R "${src}/." "${dst}/" 2>/dev/null || true
    echo "[collect_artifacts] copied dir: ${src} -> ${dst}"
  else
    echo "[collect_artifacts] skip missing dir: ${src}"
  fi
}

copy_find_matches() {
  local search_root="$1"
  local pattern="$2"
  local out_subdir="$3"
  if [ ! -d "${search_root}" ]; then
    echo "[collect_artifacts] skip missing search root: ${search_root}"
    return
  fi
  local out_dir="${STEP_DIR}/${out_subdir}"
  mkdir -p "${out_dir}"
  find "${search_root}" -type f -iname "${pattern}" -print0 2>/dev/null | while IFS= read -r -d '' file; do
    local base
    base="$(basename "${file}")"
    cp -f "${file}" "${out_dir}/${base}" 2>/dev/null || true
  done
}

copy_file_if_exists "${STDOUT_FILE}" "${STEP_DIR}/stdout.log"
copy_file_if_exists "${STDERR_FILE}" "${STEP_DIR}/stderr.log"
copy_file_if_exists "${STEP_LOG_FILE}" "${STEP_DIR}/step.log"

copy_dir_if_exists "${ROOT_DIR}/build/e2e_failures" "${STEP_DIR}/e2e_failures"
copy_dir_if_exists "${ROOT_DIR}/build/determinism_failures" "${STEP_DIR}/determinism_failures"
copy_dir_if_exists "${ROOT_DIR}/build/desktop_smoke" "${STEP_DIR}/desktop_smoke"
copy_dir_if_exists "${ROOT_DIR}/build/support_zip" "${STEP_DIR}/support_zip"

# Best-effort support zip discovery in build outputs.
copy_find_matches "${ROOT_DIR}/build" "*support*zip*" "support_zip_discovered"
# Best-effort smoke json discovery.
copy_find_matches "${ROOT_DIR}/build" "*smoke*.json" "smoke_json_discovered"

echo "[collect_artifacts] done: ${STEP_DIR}"
exit 0

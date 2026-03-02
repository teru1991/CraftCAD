#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG_DIR="${ROOT_DIR}/.ci_logs"
SUMMARY_SCRIPT="${ROOT_DIR}/scripts/ci/parse_failures.py"
SUMMARY_FILE="${LOG_DIR}/summary.json"

mkdir -p "${LOG_DIR}"
rm -f "${LOG_DIR}"/*.log "${SUMMARY_FILE}"

overall_status=0

run_step() {
  local name="$1"
  local workdir="$2"
  shift 2
  local log_file="${LOG_DIR}/${name}.log"

  echo "==> ${name}" | tee "${log_file}"
  (
    cd "${workdir}" && "$@"
  ) >>"${log_file}" 2>&1
  local status=$?

  if [ ${status} -ne 0 ]; then
    echo "[FAIL] ${name} (exit ${status})" | tee -a "${log_file}"
    overall_status=1
  else
    echo "[PASS] ${name}" | tee -a "${log_file}"
  fi
}

run_step rust_fmt "${ROOT_DIR}/core" cargo fmt --all -- --check
run_step rust_clippy "${ROOT_DIR}/core" cargo clippy --workspace --all-targets -- -D warnings
run_step rust_test "${ROOT_DIR}/core" cargo test --workspace --all-targets
run_step diycad_format_tests "${ROOT_DIR}/core" cargo test -p diycad_format --tests
run_step diycad_format_tests_latest2 "${ROOT_DIR}/core" cargo test -p diycad_format --features test_latest_2 --test migrate_applies_under_feature
run_step migration_tests "${ROOT_DIR}/core" cargo test -p migration
run_step ssot_lint "${ROOT_DIR}" cargo run -q -p ssot_lint --bin ssot-lint --manifest-path core/Cargo.toml
run_step e2e_shelf_flow "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export
run_step determinism_wizard "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test wizard_determinism
run_step compat_presets_templates "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test presets_templates_compat

if [ -f "${ROOT_DIR}/apps/desktop/CMakeLists.txt" ]; then
  run_step rust_ffi_desktop "${ROOT_DIR}/core" cargo build -p craftcad_ffi_desktop
  DESKTOP_BUILD_DIR="${ROOT_DIR}/build/desktop"
  run_step rust_ffi_build "${ROOT_DIR}/core" cargo build -p craftcad_ffi_desktop
  run_step cmake_configure "${ROOT_DIR}" cmake -S apps/desktop -B "${DESKTOP_BUILD_DIR}" -DCMAKE_BUILD_TYPE=Release
  run_step cmake_build "${ROOT_DIR}" cmake --build "${DESKTOP_BUILD_DIR}" --parallel

  if [ -f "${DESKTOP_BUILD_DIR}/CTestTestfile.cmake" ] || [ -d "${DESKTOP_BUILD_DIR}/Testing" ]; then
    run_step ctest "${ROOT_DIR}" ctest --test-dir "${DESKTOP_BUILD_DIR}" --output-on-failure
  else
    echo "==> ctest" > "${LOG_DIR}/ctest.log"
    echo "[SKIP] ctest metadata not found in ${DESKTOP_BUILD_DIR}" >> "${LOG_DIR}/ctest.log"
  fi
fi

python3 "${SUMMARY_SCRIPT}" --log-dir "${LOG_DIR}" --out "${SUMMARY_FILE}"

if [ ${overall_status} -eq 0 ]; then
  python3 - <<'PY' "${SUMMARY_FILE}"
import json, sys
summary_path = sys.argv[1]
with open(summary_path, encoding='utf-8') as f:
    summary = json.load(f)
if summary.get('total_failures', 0) != 0:
    sys.exit(1)
PY
  overall_status=$?
fi

exit ${overall_status}

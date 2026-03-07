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

run_step_expect_fail_grep() {
  local name="$1"
  local workdir="$2"
  local expect="$3"
  shift 3
  local log_file="${LOG_DIR}/${name}.log"

  echo "==> ${name}" | tee "${log_file}"
  (
    cd "${workdir}" && "$@"
  ) >>"${log_file}" 2>&1
  local status=$?

  if [ ${status} -eq 0 ]; then
    echo "[FAIL] ${name} expected failure but got success" | tee -a "${log_file}"
    overall_status=1
    return
  fi
  if ! grep -q "${expect}" "${log_file}"; then
    echo "[FAIL] ${name} missing expected pattern: ${expect}" | tee -a "${log_file}"
    overall_status=1
    return
  fi
  echo "[PASS] ${name}" | tee -a "${log_file}"
}

# Sprint14 final PR-gate checklist (must stay aligned with docs/verification/SPRINT14-STEP8.md):
# 1) ssot-lint (required files + schema hash)
# 2) diycad_format tests: unit/determinism/limits/atomic-save/schema-compat + test_latest_2 hook
# 3) migration crate tests
# 4) recovery crate tests + crash recovery e2e
# 5) golden / compat / fuzz(quick) / determinism / e2e(batch)
# 6) migrate tool tests (diycad-migrate)
run_step rust_fmt "${ROOT_DIR}/core" cargo fmt --all -- --check
run_step rust_clippy "${ROOT_DIR}/core" cargo clippy -p craftcad_wizards --all-targets -- -D warnings -A clippy::too-many-arguments -A clippy::unnecessary-sort-by
run_step rust_test "${ROOT_DIR}/core" cargo test -p craftcad_wizards --all-targets
run_step viewpack_build_verify "${ROOT_DIR}/core" cargo test -p craftcad_viewpack
run_step diycad_format_tests "${ROOT_DIR}/core" cargo test -p diycad_format --tests
run_step diycad_format_tests_latest2 "${ROOT_DIR}/core" cargo test -p diycad_format --features test_latest_2 --test migrate_applies_under_feature
run_step migration_tests "${ROOT_DIR}/core" cargo test -p migration
run_step ssot_lint "${ROOT_DIR}" cargo run -q -p ssot_lint --bin ssot-lint --manifest-path core/Cargo.toml
run_step e2e_shelf_flow "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export
run_step determinism_wizard "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test wizard_determinism
run_step compat_presets_templates "${ROOT_DIR}/core" cargo test -p craftcad_wizards --test presets_templates_compat
run_step golden_diycad_open_save "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test diycad_open_save
run_step compat_open "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test compat_open
run_step fuzz_diycad_open_short "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test diycad_open_fuzz
run_step determinism_open_signature "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test open_signature
run_step e2e_migrate_verify_batch "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test migrate_verify_batch
run_step recovery_tests "${ROOT_DIR}/core" cargo test -q -p recovery
run_step e2e_crash_recovery "${ROOT_DIR}/core" cargo test -q -p craftcad_wizards --test project_crash_recovery
run_step tools_migrate_tests "${ROOT_DIR}/tools/migrate" cargo test -q -p diycad-migrate
run_step diagnostics_tests "${ROOT_DIR}/core" cargo test -p craftcad_diagnostics --tests
run_step diagnostics_golden "${ROOT_DIR}/core/crates/ssot_lint" cargo test --test diagnostics_golden
run_step diagnostics_support_zip_e2e "${ROOT_DIR}/core/crates/ssot_lint" cargo test --test diagnostics_support_zip
run_step perf_smoke "${ROOT_DIR}" scripts/ci/perf_smoke.sh

if [ -f "${ROOT_DIR}/apps/desktop/CMakeLists.txt" ]; then
  if pkg-config --exists Qt6Core 2>/dev/null; then
    DESKTOP_BUILD_DIR="${ROOT_DIR}/build/desktop"
    DESKTOP_SMOKE_FIXTURE="${ROOT_DIR}/build/desktop/view3d_smoke_fixture.diycad"
    DESKTOP_RULES_FIXTURE="${ROOT_DIR}/build/desktop/rules_edge_smoke_fixture.diycad"
    run_step desktop_build "${ROOT_DIR}" scripts/build_desktop.sh
    run_step desktop_smoke_fixture "${ROOT_DIR}" python3 scripts/ci/create_view3d_smoke_fixture.py "${DESKTOP_SMOKE_FIXTURE}"
    run_step desktop_rules_smoke_fixture "${ROOT_DIR}" python3 scripts/ci/create_rules_edge_smoke_fixture.py "${DESKTOP_RULES_FIXTURE}"
    run_step viewpack_inspect "${ROOT_DIR}/core" cargo run -q -p craftcad_viewpack_inspect --bin craftcad-viewpack-inspect -- "${DESKTOP_RULES_FIXTURE}"
    run_step desktop_smoke_view3d "${ROOT_DIR}" ./scripts/run_desktop.sh --smoke-view3d "${DESKTOP_SMOKE_FIXTURE}"
    run_step desktop_smoke_projection_lite "${ROOT_DIR}" ./scripts/run_desktop.sh --smoke-projection-lite "${DESKTOP_SMOKE_FIXTURE}"
    run_step desktop_smoke_estimate_lite "${ROOT_DIR}" ./scripts/run_desktop.sh --smoke-estimate-lite "${DESKTOP_SMOKE_FIXTURE}"
    run_step desktop_smoke_mfg_hints_lite "${ROOT_DIR}" ./scripts/run_desktop.sh --smoke-mfg-hints-lite "${DESKTOP_RULES_FIXTURE}"
    run_step desktop_smoke_rules_edge "${ROOT_DIR}" ./scripts/run_desktop.sh --smoke-rules-edge "${DESKTOP_RULES_FIXTURE}"
    run_step_expect_fail_grep desktop_smoke_export_preflight "${ROOT_DIR}" "BLOCKED=1" ./scripts/run_desktop.sh --smoke-export-preflight "${DESKTOP_RULES_FIXTURE}"

    if [ -f "${DESKTOP_BUILD_DIR}/CTestTestfile.cmake" ] || [ -d "${DESKTOP_BUILD_DIR}/Testing" ]; then
      run_step ctest "${ROOT_DIR}" ctest --test-dir "${DESKTOP_BUILD_DIR}" --output-on-failure
    else
      echo "==> ctest" > "${LOG_DIR}/ctest.log"
      echo "[SKIP] ctest metadata not found in ${DESKTOP_BUILD_DIR}" >> "${LOG_DIR}/ctest.log"
    fi
  else
    echo "==> desktop_qt_check" > "${LOG_DIR}/desktop_qt_check.log"
    echo "[SKIP] Qt6 development package not available; skipping desktop CMake build" >> "${LOG_DIR}/desktop_qt_check.log"
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

# Always try to collect reproducible perf artifacts (non-fatal).
set +e
"${ROOT_DIR}/scripts/ci/collect_artifacts.sh" "${ROOT_DIR}/artifacts" || true
set -e

exit ${overall_status}

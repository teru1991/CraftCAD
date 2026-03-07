#!/usr/bin/env bash
set -u

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG_DIR="${ROOT_DIR}/.ci_logs"
SUMMARY_SCRIPT="${ROOT_DIR}/scripts/ci/parse_failures.py"
SUMMARY_FILE="${LOG_DIR}/summary.json"
ARTIFACTS_DIR="${ROOT_DIR}/build/ci_artifacts"
COLLECTOR_SCRIPT="${ROOT_DIR}/scripts/ci/collect_artifacts.sh"
ARTIFACTS_INDEX_SCRIPT="${ROOT_DIR}/scripts/ci/artifacts_index.py"

mkdir -p "${LOG_DIR}" "${ARTIFACTS_DIR}"
rm -f "${LOG_DIR}"/*.log "${SUMMARY_FILE}"

overall_status=0

collect_failure_artifacts() {
  local step_name="$1"
  local exit_code="$2"
  local stdout_file="$3"
  local stderr_file="$4"
  local log_file="$5"

  if [ -x "${COLLECTOR_SCRIPT}" ] || [ -f "${COLLECTOR_SCRIPT}" ]; then
    bash "${COLLECTOR_SCRIPT}" "${step_name}" "${exit_code}" "${stdout_file}" "${stderr_file}" "${log_file}" || true
  else
    echo "[ARTIFACT] collector missing: ${COLLECTOR_SCRIPT}" >&2
  fi
}

run_step() {
  local name="$1"
  local workdir="$2"
  shift 2
  local log_file="${LOG_DIR}/${name}.log"
  local stdout_file
  local stderr_file
  stdout_file="$(mktemp "${LOG_DIR}/${name}.stdout.XXXXXX")"
  stderr_file="$(mktemp "${LOG_DIR}/${name}.stderr.XXXXXX")"

  echo "==> ${name}" | tee "${log_file}"
  (
    cd "${workdir}" && "$@"
  ) >"${stdout_file}" 2>"${stderr_file}"
  local status=$?

  cat "${stdout_file}" >>"${log_file}" 2>/dev/null || true
  cat "${stderr_file}" >>"${log_file}" 2>/dev/null || true

  if [ ${status} -ne 0 ]; then
    echo "[FAIL] ${name} (exit ${status})" | tee -a "${log_file}"
    collect_failure_artifacts "${name}" "${status}" "${stdout_file}" "${stderr_file}" "${log_file}"
    overall_status=1
  else
    echo "[PASS] ${name}" | tee -a "${log_file}"
  fi

  rm -f "${stdout_file}" "${stderr_file}"
}

run_step_skip() {
  local name="$1"
  local reason="$2"
  local log_file="${LOG_DIR}/${name}.log"
  echo "==> ${name}" | tee "${log_file}"
  echo "[SKIP] ${reason}" | tee -a "${log_file}"
}

run_step_expect_fail_grep() {
  local name="$1"
  local workdir="$2"
  local expect="$3"
  shift 3
  local log_file="${LOG_DIR}/${name}.log"
  local stdout_file
  local stderr_file
  stdout_file="$(mktemp "${LOG_DIR}/${name}.stdout.XXXXXX")"
  stderr_file="$(mktemp "${LOG_DIR}/${name}.stderr.XXXXXX")"

  echo "==> ${name}" | tee "${log_file}"
  (
    cd "${workdir}" && "$@"
  ) >"${stdout_file}" 2>"${stderr_file}"
  local status=$?

  cat "${stdout_file}" >>"${log_file}" 2>/dev/null || true
  cat "${stderr_file}" >>"${log_file}" 2>/dev/null || true

  if [ ${status} -eq 0 ]; then
    echo "[FAIL] ${name} expected failure but got success" | tee -a "${log_file}"
    collect_failure_artifacts "${name}" "${status}" "${stdout_file}" "${stderr_file}" "${log_file}"
    overall_status=1
    rm -f "${stdout_file}" "${stderr_file}"
    return
  fi
  if ! grep -q "${expect}" "${log_file}"; then
    echo "[FAIL] ${name} missing expected pattern: ${expect}" | tee -a "${log_file}"
    collect_failure_artifacts "${name}" "${status}" "${stdout_file}" "${stderr_file}" "${log_file}"
    overall_status=1
    rm -f "${stdout_file}" "${stderr_file}"
    return
  fi
  echo "[PASS] ${name}" | tee -a "${log_file}"
  rm -f "${stdout_file}" "${stderr_file}"
}

require_cmd() {
  local cmd="$1"
  local _reason="$2"
  command -v "${cmd}" >/dev/null 2>&1
}

detect_qt6() {
  if require_cmd pkg-config "pkg-config unavailable" && pkg-config --exists Qt6Core 2>/dev/null; then
    return 0
  fi
  if require_cmd cmake "cmake unavailable" && cmake --find-package -DNAME=Qt6Core -DCOMPILER_ID=GNU -DLANGUAGE=CXX -DMODE=EXIST >/dev/null 2>&1; then
    return 0
  fi
  if require_cmd qmake6 "qmake6 unavailable" && qmake6 -v 2>/dev/null | grep -q "Qt version 6"; then
    return 0
  fi
  return 1
}

run_step crossplatform_checks "${ROOT_DIR}" python3 scripts/ci/crossplatform_checks.py

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
run_step determinism_harness "${ROOT_DIR}/core" cargo run -q -p craftcad_determinism_harness --bin craftcad-determinism-check
run_step dirty_deps_tests "${ROOT_DIR}" cargo test -p craftcad_dirty_deps --manifest-path core/Cargo.toml
run_step dirty_engine_tests "${ROOT_DIR}" cargo test -p craftcad_dirty_engine --manifest-path core/Cargo.toml
run_step params_registry_tests "${ROOT_DIR}" cargo test -p craftcad_params_registry --manifest-path core/Cargo.toml
run_step artifact_store_tests "${ROOT_DIR}" cargo test -p craftcad_artifact_store --manifest-path core/Cargo.toml
run_step regen_scheduler_tests "${ROOT_DIR}" cargo test -p craftcad_regen_scheduler --manifest-path core/Cargo.toml
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

if [ ! -f "${ROOT_DIR}/apps/desktop/CMakeLists.txt" ]; then
  run_step_skip desktop_gate "apps/desktop/CMakeLists.txt not found"
elif ! detect_qt6; then
  run_step_skip desktop_gate "Qt6 tooling not detected; desktop build+smoke gate skipped"
else
  DESKTOP_BUILD_DIR="${ROOT_DIR}/build/desktop"
  DESKTOP_SMOKE_JSON_DIR="${ROOT_DIR}/build/desktop_smoke"
  DESKTOP_SMOKE_FIXTURE="${ROOT_DIR}/build/desktop/view3d_smoke_fixture.diycad"
  DESKTOP_RULES_FIXTURE="${ROOT_DIR}/build/desktop/rules_edge_smoke_fixture.diycad"
  SMOKE_VALIDATOR="${ROOT_DIR}/scripts/ci/validate_smoke_json.py"

  mkdir -p "${DESKTOP_SMOKE_JSON_DIR}"

  run_step desktop_build "${ROOT_DIR}" scripts/build_desktop.sh
  run_step desktop_print_env "${ROOT_DIR}" bash ./scripts/run_desktop.sh --print-env
  run_step desktop_smoke_resources "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-resources
  run_step desktop_smoke_resources_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_resources.log" --expect-token "\"name\":\"resources\"" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_resources.json"
  run_step desktop_smoke_fixture "${ROOT_DIR}" python3 scripts/ci/create_view3d_smoke_fixture.py "${DESKTOP_SMOKE_FIXTURE}"
  run_step desktop_rules_smoke_fixture "${ROOT_DIR}" python3 scripts/ci/create_rules_edge_smoke_fixture.py "${DESKTOP_RULES_FIXTURE}"

  if [ -x "${ROOT_DIR}/core/target/debug/craftcad-viewpack-inspect" ] || [ -f "${ROOT_DIR}/core/crates/craftcad_viewpack_inspect/Cargo.toml" ]; then
    run_step viewpack_inspect "${ROOT_DIR}/core" cargo run -q -p craftcad_viewpack_inspect --bin craftcad-viewpack-inspect -- "${DESKTOP_RULES_FIXTURE}"
  else
    run_step_skip viewpack_inspect "craftcad_viewpack_inspect crate not present"
  fi

  run_step desktop_smoke_view3d "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-view3d "${DESKTOP_SMOKE_FIXTURE}"
  run_step desktop_smoke_view3d_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_view3d.log" --expect-token "VIEW3D_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_view3d.json"

  run_step desktop_smoke_projection_lite "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-projection-lite "${DESKTOP_SMOKE_FIXTURE}"
  run_step desktop_smoke_projection_lite_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_projection_lite.log" --expect-token "PROJ_LITE_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_projection_lite.json"

  run_step desktop_smoke_estimate_lite "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-estimate-lite "${DESKTOP_SMOKE_FIXTURE}"
  run_step desktop_smoke_estimate_lite_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_estimate_lite.log" --expect-token "ESTIMATE_LITE_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_estimate_lite.json"

  run_step desktop_smoke_fastener_bom_lite "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-fastener-bom-lite "${DESKTOP_RULES_FIXTURE}"
  run_step desktop_smoke_fastener_bom_lite_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_fastener_bom_lite.log" --expect-token "FASTENER_BOM_LITE_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_fastener_bom_lite.json"

  run_step desktop_smoke_rules_edge "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-rules-edge "${DESKTOP_RULES_FIXTURE}"
  run_step desktop_smoke_rules_edge_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_rules_edge.log" --expect-token "RULES_EDGE_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_rules_edge.json"

  run_step desktop_smoke_mfg_hints_lite "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-mfg-hints-lite "${DESKTOP_RULES_FIXTURE}"
  run_step desktop_smoke_mfg_hints_lite_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_mfg_hints_lite.log" --expect-token "MFG_HINTS_LITE_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_mfg_hints_lite.json"

  run_step desktop_smoke_inspector_edit "${ROOT_DIR}" bash ./scripts/run_desktop.sh --smoke-inspector-edit "${DESKTOP_RULES_FIXTURE}"
  run_step desktop_smoke_inspector_edit_json "${ROOT_DIR}" python3 "${SMOKE_VALIDATOR}" --log "${LOG_DIR}/desktop_smoke_inspector_edit.log" --expect-token "INSPECTOR_SMOKE_OK" --out "${DESKTOP_SMOKE_JSON_DIR}/desktop_smoke_inspector_edit.json"

  run_step_expect_fail_grep desktop_smoke_export_preflight "${ROOT_DIR}" "BLOCKED=1" bash ./scripts/run_desktop.sh --smoke-export-preflight "${DESKTOP_RULES_FIXTURE}"

  if [ -f "${DESKTOP_BUILD_DIR}/CTestTestfile.cmake" ] || [ -d "${DESKTOP_BUILD_DIR}/Testing" ]; then
    run_step ctest "${ROOT_DIR}" ctest --test-dir "${DESKTOP_BUILD_DIR}" --output-on-failure
  else
    run_step_skip ctest "ctest metadata not found in ${DESKTOP_BUILD_DIR}"
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

# Always generate artifact index (even if empty).
python3 "${ARTIFACTS_INDEX_SCRIPT}" --artifacts-dir "${ARTIFACTS_DIR}" >/dev/null 2>&1 || true

exit ${overall_status}

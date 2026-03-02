# Codex Red/Green Loop

## Purpose
This repository standardizes a single CI entrypoint so Codex (and humans) can iterate in a fixed loop:

1. Run `scripts/ci/run_all.sh` (or `scripts/ci/run_all.ps1` on Windows)
2. Read `.ci_logs/summary.json`
3. Fix the smallest root cause
4. Re-run `run_all`
5. Repeat until no failures remain

## Entrypoints
- Linux/macOS: `scripts/ci/run_all.sh`
- Windows: `scripts/ci/run_all.ps1`

Both scripts emit per-step logs into `.ci_logs/*.log` and a machine-readable summary at `.ci_logs/summary.json`.

## Failure categorization
`parse_failures.py` classifies failures into these categories:
- `rust_compile_error`
- `clippy`
- `rust_test_fail`
- `link_error`
- `cmake_error`
- `qt_meta_object`
- `ctest_fail`

The summary also proposes the first fix priority by this order: `fmt -> clippy -> test -> link -> desktop`.

## Pitfalls and mitigations observed in this implementation
- **Problem:** Stopping at the first failed command loses later diagnostics.
  - **Mitigation:** `run_all` captures each step into its own log file and continues executing subsequent checks.
- **Problem:** Agents can mark work done without checking aggregate state.
  - **Mitigation:** `run_all` returns non-zero for any step failure and guards against non-zero `summary.json` failure counts.
- **Problem:** Desktop checks are not always present in every environment.
  - **Mitigation:** Desktop configure/build/test steps only run when a desktop CMake project exists; missing CTest metadata is logged as a skip.

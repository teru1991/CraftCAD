# G1-6-TRACE-VERIFICATION-ALIGN-001 Verification

## Summary
- Added a minimal trace contract to formalize `docs/status/trace-index.json` expectations.
- Aligned `trace-index` with existing verification docs so every verification file is referenced and every task has `verification` + existing `artifacts` paths.
- Added CI checker `scripts/ci/trace_index_check.py` and wired it as an early gate in `scripts/ci/run_all.sh`.

## Changed files
- `docs/status/trace-index.json`
- `docs/status/trace-contract.md`
- `scripts/ci/trace_index_check.py`
- `scripts/ci/run_all.sh`
- `docs/verification/G1-6-TRACE-VERIFICATION-ALIGN-001.md`

## History evidence
- `git status --porcelain`: clean before changes.
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-6-trace-verification-align-001`: success.
- `git rev-parse HEAD`: `65a7242bfea01dbe1a70f9746847ce73a1fb0dc1`.
- `git log -n 40 --oneline`: reviewed.
- `git branch -vv`: confirmed active branch.

## Inventory / scan outputs
- `docs/status` and `docs/verification` inventory confirmed `trace-index.json` plus 61 verification files.
- Initial scan:
  - `tasks: 56`
  - `missing verification: 0`
  - `verification files: 61`
  - `orphans: 11`
  - orphan files:
    - `S12-IO-BRIDGE-004.md`
    - `S12-IO-COMPAT-005.md`
    - `S12-IO-DXF-003.md`
    - `S12-IO-SSOT-001.md`
    - `S12-IO-SVG-002.md`
    - `S16-JOBS-QUEUE-003.md`
    - `S19-B20-PR1.md`
    - `S19-B20-PR3.md`
    - `S19-B20-PR4.md`
    - `S20-STEP4.md`
    - `SPRINT14-STEP7.md`

## What was fixed
- Added missing `verification` field for:
  - `S12-IO-SSOT-001`
  - `S12-IO-SVG-002`
  - `S12-IO-DXF-003`
  - `S12-IO-BRIDGE-004`
  - `S12-IO-COMPAT-005`
- Added missing task entries:
  - `S16-JOBS-QUEUE-003`
  - `S19-B20-PR1`
  - `S19-B20-PR3`
  - `S19-B20-PR4`
  - `S20-STEP4`
- Fixed `SPRINT14-STEP7` verification reference (`verification_evidence` string -> `verification`).
- Replaced non-existent wildcard/placeholder artifact paths with existing concrete paths to satisfy existence checks.
- Final scan:
  - `tasks: 61`
  - `missing verification: 0`
  - `orphans: 0`

## Local verification
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.
- `python3 scripts/ci/trace_index_check.py`: pass (`[trace-check] OK tasks=61 verifications=61`).
- `./scripts/ci/run_all.sh`:
  - `trace_index_check` step passes early.
  - Existing unrelated failures remain (`rust_test`, `e2e_shelf_flow`) after trace check.

## Self-check
- Allowlist respected (`docs/**`, `scripts/**` only).
- No deletions performed.

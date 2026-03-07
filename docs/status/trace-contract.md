# Trace Contract

## Purpose
`docs/status/trace-index.json` is the SSOT for task traceability between branch, artifacts, and verification evidence.

## Contract
For every task entry under `tasks`:
- `branch` is required.
- `verification` is required and must point to an existing file.
- `artifacts` is required, non-empty, and every listed path must exist.
- `status` is optional (`planned | in_progress | done`) and only used if explicitly needed.

## Naming
- Verification file: `docs/verification/<TASK-ID>.md`
- Branch (recommended): `feature/<task-id-lower>-001`

## Rules
- Do not delete existing task entries.
- If a task is obsolete, keep the entry and mark deprecation in notes/evidence fields instead of deleting artifact history.

## CI Check
- `scripts/ci/trace_index_check.py` validates:
  - JSON validity
  - `verification` file existence
  - `artifacts` list presence and path existence
  - duplicate verification references

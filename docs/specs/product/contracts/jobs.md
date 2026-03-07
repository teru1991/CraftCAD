# Jobs Contract (Progress/Cancel/Determinism)

## Job invariants
- Every heavy operation is a Job.
- A Job must provide:
  - `job_id`, `job_type`, `created_at`
  - progress (stage or percent)
  - cancel capability (if safe)
  - result link (open artifact / show report)
  - failure ReasonCode + next actions

## Canonical job types (minimum)
1) `JOB_REGEN_3D_VIEW`
- Input: SSOT snapshot
- Output: 3D view artifact
- Timeout: configurable (default small)
- Determinism: strict

2) `JOB_REGEN_2D_DRAWINGS`
- Input: SSOT snapshot + drawing presets
- Output: 2D sheets
- Timeout: moderate
- Determinism: strict

3) `JOB_RUN_NESTING`
- Input: parts outlines + material constraints + seed + heuristics config
- Output: nesting sheets + cutlist + yield
- Timeout: moderate; cancel supported
- Determinism: strict w/ seed and stable ordering

4) `JOB_EXPORT`
- Input: chosen artifacts + export target (PDF/DXF/SVG/JSON)
- Output: export files + manifest
- Timeout: moderate; cancel supported
- Determinism: strict

5) `JOB_BUILD_STEPS`
- Input: SSOT snapshot + screw features + rules config
- Output: steps + manufacturing hints
- Timeout: small; deterministic

6) `JOB_DETERMINISM_CHECK`
- Input: same SSOT snapshot (+ fixed seed/version context)
- Output: lite-artifact hash report (`ProjectionLite`/`EstimateLite`/`FastenerBOMLite`)
- Timeout: small
- Determinism: must run hashes multiple times on identical input and require full equality

## Job failure requirements
- Never crash silently.
- Must return:
  - `ReasonCode` (machine readable)
  - human summary + fix hints (UI layer)
  - optional diagnostic attachments (SupportZip integration; see ../diagnostics/)

Preflight rules must run before export, nesting-confirm, and regen jobs that emit manufacturing outputs.
If any preflight finding is `FATAL`, the job must fail with its `ReasonCode` and no output artifact is produced.

## Links
- Determinism SSOT: ../determinism/
- Diagnostics SSOT: ../diagnostics/
- UX job flow SSOT: ../ux/

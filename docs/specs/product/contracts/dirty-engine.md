# Dirty Engine Contract (SSOT)

The dirty engine consumes SSOT change kinds and returns a deterministic dirty plan for regeneration scheduling.
Mapping source of truth is `dirty-deps.md`.

## Input
- `change_set: Vec<ChangeKind>`
  - duplicates allowed

## Output
- `DirtyPlanV1`
  - `schema_version = 1`
  - `dirty_artifacts`: ordered list of
    - `{ artifact_kind, reasons: [ChangeKind...] }`

## Rules
- Union invalidations from all changes using `dirty-deps` mapping.
- Unknown/future changes produce no invalidation (forward-compatible behavior).
- Reasons are unique and sorted.

## Determinism
- Normalize input changes: dedup + sort.
- Order output artifacts deterministically by `ArtifactKind` enum order.

## Consumer
This plan is consumed by regeneration scheduler/job planner in subsequent steps.

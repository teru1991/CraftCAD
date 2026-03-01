# Nesting Edge Cases (v1)

- Time/iteration stop returns best-so-far result when available.
- Stop reasons are explicit (`NEST_STOPPED_BY_TIME_LIMIT`, `NEST_STOPPED_BY_ITERATION_LIMIT`).
- Per-part failures must include actionable reason code.
- Deterministic ordering:
  - parts: area desc, span desc, id asc
  - sheets: definition order
  - placements: deterministic candidate traversal and tie-breaks.
- If multiple failure causes apply, highest-priority reason is selected deterministically by policy order.

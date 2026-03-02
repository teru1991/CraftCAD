# Migration policy (SSOT)

Compatibility contract
- Loader MUST support reading N-2 (latest, latest-1, latest-2).
- Forward version (schema_version > latest): may open read-only best-effort if allow_forward_compat_readonly=true.
- Too old (schema_version < latest-2): may attempt best-effort open, but MUST surface ReasonCode suggesting migration tool.

Migration contract
- Migration is always stepwise: vN -> vN+1 (no skipping).
- Each step:
  1) schema validate (input)
  2) transform (pure, deterministic)
  3) schema validate (output)
  4) logical validate (output)
- Migration must be deterministic:
  - stable ordering, stable rounding, no nondeterministic iteration.

Breaking changes
- Any breaking change requires:
  - versions.md update
  - new migration step implementation
  - compat assets update (tests/compat)
  - release note entry (out of this Sprint scope)

Dry-run report
- migration tool can generate a dry-run report with stable ordering (path order).
- It must include:
  - schema_version: N -> N+1
  - added/removed/changed fields (JSON pointer paths)
  - counts: parts/nest_jobs/assets delta

# ReasonCode Contract (Minimum Set for the 6 Pillars)

## Principles
- ReasonCodes are stable identifiers.
- They map to localized human messages (see ../i18n/).
- They include optional fix hints and affected IDs.

## Reference/Sync (Pillar 1)
- `REF_TARGET_MISSING`
- `REF_AMBIGUOUS_MATCH`
- `REGEN_TIMEOUT`
- `MODEL_INVALID_AFTER_EDIT`

## Nesting/BOM (Pillar 2)
- `MATERIAL_MISSING_FOR_PART`
- `NEST_NO_FEASIBLE_LAYOUT`
- `NEST_GRAIN_CONSTRAINT_CONFLICT`
- `NEST_TIMEOUT`

## Safety rules (Pillar 3)
- `RULE_EDGE_DISTANCE_VIOLATION`
- `RULE_SCREW_BREAKTHROUGH`
- `RULE_THICKNESS_MISMATCH`
- `RULE_HOLE_SPACING_VIOLATION`
- `RULE_GRAIN_POLICY_VIOLATION`

## Build steps (Pillar 4)
- `PLAN_INCOMPLETE_DATA`
- `PLAN_UNSUPPORTED_FEATURE`
- `PLAN_CONFLICTING_CONSTRAINTS`

## Mobile viewer pack (Pillar 5)
- `VIEWPACK_MISSING_ARTIFACT`
- `VIEWPACK_HASH_MISMATCH`
- `VIEWPACK_UNSUPPORTED_VERSION`

## Quality/Diagnostics (Pillar 6)
- `JOB_CANCELLED`
- `DIAG_SUPPORTZIP_REDACTION_FAILED` (must degrade safely)

## Links
- Errors SSOT: ../errors/
- Diagnostics SSOT: ../diagnostics/

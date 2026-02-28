# Reason Codes

Transform editing and desktop tools use these edit-specific reason codes:

- `EDIT_NO_SELECTION`: transform command was invoked with an empty selection.
- `EDIT_TARGET_LOCKED_OR_HIDDEN`: at least one target entity is on a hidden/locked/non-editable layer.
- `EDIT_INVALID_NUMERIC`: invalid numeric input for transform parameters.
- `EDIT_TRANSFORM_WOULD_DEGENERATE`: transform would collapse geometry (ex: zero scale).

- `GEOM_OFFSET_SELF_INTERSECTION`: offset would create invalid/self-intersecting result.
- `GEOM_OFFSET_NOT_SUPPORTED`: requested offset is not supported for this geometry in v1.
- `GEOM_TRIM_NO_INTERSECTION`: trim/extend target does not intersect cutter.
- `EDIT_AMBIGUOUS_TARGET`: target selection is ambiguous.
- `EDIT_TRIM_AMBIGUOUS_CANDIDATE`: trim produced multiple equally valid candidates; user must choose.

# Reason Codes

Transform editing and desktop tools use these edit-specific reason codes:

- `EDIT_NO_SELECTION`: transform command was invoked with an empty selection.
- `EDIT_TARGET_LOCKED_OR_HIDDEN`: at least one target entity is on a hidden/locked/non-editable layer.
- `EDIT_INVALID_NUMERIC`: invalid numeric input for transform parameters.
- `EDIT_TRANSFORM_WOULD_DEGENERATE`: transform would collapse geometry (ex: zero scale).

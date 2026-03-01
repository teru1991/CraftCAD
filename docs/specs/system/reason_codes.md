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

- `FACE_NO_CLOSED_LOOP`: no valid closed loop was found for face extraction.
- `FACE_SELF_INTERSECTION`: closed loop self-intersects and cannot form a valid face.
- `FACE_AMBIGUOUS_LOOP`: loop containment is ambiguous (touching boundaries/epsilon ambiguity).
- `PART_INVALID_OUTLINE`: provided face/outline is invalid for part creation.

- `PART_INVALID_FIELDS`: part properties are invalid (quantity/thickness/margin/kerf/grain policy).
- `MATERIAL_NOT_FOUND`: part references missing material id in project catalog.
- `BOM_EXPORT_FAILED`: BOM serialization/export failed.

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


- `NEST_PART_TOO_LARGE_FOR_ANY_SHEET`: part (with effective margin/kerf) cannot fit in any sheet orientation.
- `NEST_GRAIN_CONSTRAINT_BLOCKS_FIT`: grain policy restricts orientations such that no feasible placement exists.
- `NEST_NO_FEASIBLE_POSITION_WITH_MARGIN_AND_KERF`: no collision-free position exists after margin/kerf inflation.
- `NEST_NO_GO_ZONE_BLOCKS_FIT`: all candidate positions are blocked by no-go zones.
- `NEST_STOPPED_BY_TIME_LIMIT`: optimization stopped due to time limit; best-so-far returned.
- `NEST_STOPPED_BY_ITERATION_LIMIT`: optimization stopped due to iteration limit; best-so-far returned.
- `NEST_INTERNAL_INFEASIBLE`: internal consistency detected infeasible state (debug-heavy).


- `EXPORT_PDF_FAILED`: PDF generation failed for current document/options.
- `EXPORT_UNSUPPORTED_ENTITY`: export encountered a geometry/entity type not supported by v1 exporter.
- `EXPORT_UNSUPPORTED_FEATURE`: export option/feature not supported in v1.
- `EXPORT_IO_PARSE_FAILED`: export input/options JSON parse failed.
- `EXPORT_IO_WRITE_FAILED`: file write failed in host UI/export flow.


- `DRAW_INVALID_NUMERIC`: drawing tool numeric input is invalid (NaN/Inf/<=0 where forbidden).
- `DRAW_CONSTRAINT_CONFLICT`: active drawing constraints conflict and no unique solution exists.
- `DRAW_INSUFFICIENT_INPUT`: drawing commit attempted before required points/parameters were provided.


- `EDIT_FILLET_RADIUS_TOO_LARGE`: requested fillet radius cannot be realized for selected segments.
- `EDIT_CHAMFER_DISTANCE_TOO_LARGE`: chamfer distance exceeds feasible trim on selected segments.
- `EDIT_MIRROR_AXIS_INVALID`: mirror axis is invalid (degenerate or non-finite).
- `EDIT_PATTERN_INVALID_PARAMS`: pattern parameters are invalid (count/step/etc).
- `EDIT_AMBIGUOUS_CANDIDATE`: operation has multiple valid candidates and user selection is required.
- `EDIT_CANDIDATE_NOT_FOUND`: selected candidate index/reference no longer exists.

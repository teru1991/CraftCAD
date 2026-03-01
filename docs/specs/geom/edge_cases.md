# Geometry Edge Case Policy (v1)

- Parallel non-colinear lines -> `GEOM_NO_INTERSECTION`.
- Colinear overlap -> `GEOM_INTERSECTION_AMBIGUOUS` with debug `case=colinear_overlap`.
- Tangent intersections are represented as 1 point (`classification=tangent_or_single`).
- Arc range uses normalized angles in (-pi, pi] with `ccw` sweep semantics.
- Epsilon usage:
  - `eq_dist`: dedupe/equality
  - `snap_dist`: point-on-geom checks
  - `intersect_tol`: intersection tolerance and staged fallback
  - `area_tol`: area-related tolerance
- Fallback behavior:
  - deterministic tolerance relaxation stages
  - debug trail includes `GEOM_NUMERIC_UNSTABLE_FALLBACK_USED`
  - if exhausted -> `GEOM_FALLBACK_LIMIT_REACHED`.

# Materials (In-Project Catalog)

Materials are stored directly in `Document.materials` in `.diycad` (no external DB).

## Material fields
- `id` (UUID, required)
- `name` (string, required, non-empty)
- `category` (`wood` | `leather` | `other`)
- `thickness_mm` (number|null): recommended for sheet-like materials
- `sheet_default` (object|null): `{width,height,quantity}` defaults for nesting inputs
- `notes` (string, optional, may be empty)

## Constraints
- `id` must be valid UUID.
- `name` should be user-meaningful and unique in project scope (soft rule).
- `thickness_mm`, `width`, `height` must be `> 0` when provided.
- `quantity` in `sheet_default` must be `>= 1`.

## ID generation
- Material IDs are UUID v4 generated at creation time.

## Backward compatibility
- For v1 documents missing `materials`, loader normalization injects `[]`.

# Mobile Annotation Format v1

## Decision

Mobile annotations are stored in a separate **`.diycad-note` zip package**.

Reason:
- `.diycad` remains source of truth and backward-compatible with older desktop versions.
- Mobile viewer/annotation layer is isolated and cannot corrupt main CAD document.

## Package layout

- `manifest.json`
- `data/annotations.json`

### `manifest.json`

```json
{
  "kind": "diycad-note",
  "version": 1,
  "doc_id": "<uuid>"
}
```

### `data/annotations.json`

```json
{
  "doc_id": "<uuid>",
  "annotations": [
    {
      "id": "<string>",
      "page": "sheet-0",
      "points": [{"x": 10.0, "y": 5.0}],
      "text": "note text",
      "color": "#E11D48",
      "created_at": "2026-03-01T00:00:00Z"
    }
  ]
}
```

## Annotation object schema

- `id`: stable annotation id.
- `page`: target sheet/page key (`sheet-*` in v1).
- `points`: polyline-like point list.
- `text`: user-entered note text.
- `color` (optional): hex color string.
- `created_at`: UTC ISO-8601 timestamp.

## Compatibility guarantees

- Desktop versions unaware of `.diycad-note` remain unaffected.
- Mobile writes no changes into `data/document.json`.

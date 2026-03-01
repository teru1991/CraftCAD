# CraftCAD Mobile (Flutter)

## Architecture choice (locked)

**Option B** is used for v1:
- Flutter parses `.diycad` package JSON directly for viewing (`Line`/`Polyline`/`Circle`/`Arc` + nesting placements when present).
- Mobile does not mutate `data/document.json`.
- Annotation data is stored in a separate `.diycad-note` zip package to keep desktop backward compatibility.
- Share/export uses pre-generated export artifacts (`.pdf`/`.svg`) found inside `.diycad` package files.

This keeps mobile viewer-safe and avoids any risk of corrupting source-of-truth `.diycad` files.

## Supported v1 features

- Open `.diycad` and render supported drawing entities deterministically.
- Render nesting placement bounding boxes when `jobs[].result.placements[].bbox` is present.
- Add tap-based annotation points in a mobile-only layer.
- Save annotations as `<doc_id>.diycad-note` zip package.
- Share first available embedded `.pdf` or `.svg` export.

## Run

```bash
cd apps/mobile
flutter pub get
flutter run
```

## Annotation compatibility

`.diycad-note` contains:
- `manifest.json`
- `data/annotations.json`

and never modifies the original `.diycad` document.

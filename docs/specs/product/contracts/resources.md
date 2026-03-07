# Resources Contract (Desktop)

## ResourceRoot discovery order
1. `CRAFTCAD_RESOURCE_ROOT` environment variable (**absolute path only**).
2. Executable-relative path: `<exe_dir>/resources`.
3. Dev fallback (repo checkout only): `<repo_root>/apps/desktop/resources`.
   - Enabled only when repository root is detected by traversing from `exe_dir` and finding `.git`.

## Required subpaths (Step 1)
- `templates/` (required)
- `samples/` (required)
- `fonts/` (optional)
- `icons/` (optional)

## Resources preflight smoke
Desktop must expose:
- `--smoke-resources`

The smoke prints one-line JSON:
- `kind: "smoke"`
- `name: "resources"`
- `ok: true/false`
- `reason_code`: `RESOURCE_ROOT_NOT_FOUND` or `RESOURCE_MISSING` when failing
- `message`
- `evidence.root`
- `evidence.missing` (missing required subpaths)

Exit code:
- `0` when resources preflight passes
- `1` when resources preflight fails

## Failure mapping
- `RESOURCE_ROOT_NOT_FOUND`: no valid resource root was found, or env override is invalid.
- `RESOURCE_MISSING`: resource root exists but required entries are missing.

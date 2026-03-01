# Shippable v1 Operations

## Diagnostic Pack

- Exported from desktop Export menu as `diagnostic_pack.zip`.
- Includes:
  - `reason_logs/latest.json` (latest N command/FFI reasons)
  - `env.json` (os/arch/app_version/git_hash)
  - `replay.json` (seed/eps/settings_digest payload)
  - optional `snapshot/document.json` only when user explicitly opts in
- Privacy default: project snapshot is **excluded** by default.

## Crash-safe autosave

- Desktop writes autosave snapshot to app-local path every 5 seconds and on app exit.
- On startup, if snapshot exists, user is prompted to restore recovery state.

## Performance

- Selection/hittest uses uniform-grid spatial candidate pruning before geometric projection.
- Canvas rendering uses pixmap render cache invalidated by document revision/zoom changes.
- Geometry perf smoke threshold test exists in core for intersection hot path.

## Packaging / CI

- CI runs Rust fmt/clippy/tests, then desktop build (Qt6 + CMake) and uploads artifact.
- Local script: `scripts/build_desktop.sh`.

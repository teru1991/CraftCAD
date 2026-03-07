# CraftCAD Desktop

Qt6 desktop skeleton that opens `.diycad`, renders entities, and routes editing through Rust commands/history over C ABI.

## Features in this skeleton
- Open `.diycad` via file picker or CLI path
- Parse Rust FFI result envelope `{ok,data,reason}`
- Render Line entities
- Selection via hit-test (project-point through Rust geom FFI)
- Snap candidates (endpoint/midpoint/intersection)
- Line tool with preview/commit/cancel
- Constraints: `H` (horizontal), `V` (vertical)
- Numeric input while drawing (Enter/Backspace/Escape)
- Undo/redo through Rust history (`Ctrl+Z`, `Ctrl+Y`, `Ctrl+Shift+Z`)

## Prerequisites

- Qt6 development packages (e.g. `Qt6Core`, `Qt6Widgets`)
- CMake >= 3.21
- Rust toolchain (see `rust-toolchain.toml` in repo root)

## Build (official)

From this directory (`apps/desktop`):

```bash
../../scripts/build_desktop.sh
```

From repository root:

```bash
./scripts/build_desktop.sh
```

The official route builds Rust FFI (`craftcad_ffi_desktop`) in **release**, then builds the desktop app in **release** with `FFI_LIB_DIR` fixed to `core/target/release`. For normal usage, manual `-DFFI_LIB_DIR=...` is not required.

## Run (official)

From this directory (`apps/desktop`):

```bash
../../scripts/run_desktop.sh
```

From repository root:

```bash
bash ./scripts/run_desktop.sh /path/to/project.diycad
```

All desktop execution routes (including smoke flags such as `--smoke-view3d`) must use `bash ./scripts/run_desktop.sh [args...]`.

Build artifacts and runtime shared libraries under `build/` and `target/` are local/CI runtime outputs only and must not be committed to Git.

## Runtime troubleshooting

- Missing shared libraries: `scripts/run_desktop.sh` runs `ldd` (Linux) or provides `otool -L` diagnostics (macOS) and prints actionable hints.
- Qt platform plugin errors: `scripts/run_desktop.sh` prints best-effort `QT_PLUGIN_PATH` candidates when it detects plugin-load failures.
- Qt6 missing (`pkg-config --exists Qt6Core` fails): install Qt6 dev packages. CI may skip desktop build when Qt6 is unavailable.
- `craftcad_ffi_desktop` build fails: run `cargo build --release -p craftcad_ffi_desktop` in `core/` and fix Rust-side errors first.
- CMake cannot find desktop FFI library: verify `core/target/release` exists. For custom layouts only, pass `-DFFI_LIB_DIR=<path>` manually.


## Drawing tools (v1)
Tool hotkeys:
- `1`: Line
- `7`: Rect (TwoPoint mode)
- `8`: Circle (CenterRadius mode)
- `9`: Arc (Center mode)
- `0`: Polyline

Common key bindings:
- Numeric typing: builds numeric buffer for active stage
- `Enter`: commit current stage/tool
- `Tab`: cycle numeric field (v1 tools mostly single-field; retained for consistency)
- `Esc`: cancel current stage/tool
- Constraints: `H` horizontal lock, `V` vertical lock, `A` angle lock (Arc)
- Polyline: `C` close+commit, `Backspace` remove last point

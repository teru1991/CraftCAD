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

## Build

### 1) Build Rust FFI cdylib
```bash
cd core/ffi_desktop
cargo build
```

### 2) Build desktop app
From repo root:
```bash
cmake -S apps/desktop -B build/desktop -DFFI_LIB_DIR=$(pwd)/core/target/debug
cmake --build build/desktop
```

### 3) Library path
Linux:
```bash
export LD_LIBRARY_PATH=$(pwd)/core/target/debug:$LD_LIBRARY_PATH
```
macOS:
```bash
export DYLD_LIBRARY_PATH=$(pwd)/core/target/debug:$DYLD_LIBRARY_PATH
```
Windows (PowerShell):
```powershell
$env:PATH = "$(Get-Location)\core\target\debug;" + $env:PATH
```

### 4) Run
```bash
./build/desktop/craftcad_desktop /path/to/project.diycad
```


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

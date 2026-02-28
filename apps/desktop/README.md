# CraftCAD Desktop (Qt6, read-only skeleton)

This is a minimal Qt6 desktop app that opens a `.diycad` file, asks Rust to load/validate it, and renders line entities.

## What it does
- Open project by CLI arg or file picker
- Load via Rust C-ABI (`craftcad_ffi_desktop`)
- Parse returned document JSON
- Render `Line` entities in a `QGraphicsView`
- Pan: **middle mouse drag**
- Zoom: **mouse wheel**

## Build steps

### 1) Build Rust FFI library
From repository root:

```bash
cd core
cargo build -p craftcad_ffi_desktop --release
```

This produces:
- Linux: `core/target/release/libcraftcad_ffi_desktop.so`
- macOS: `core/target/release/libcraftcad_ffi_desktop.dylib`
- Windows: `core/target/release/craftcad_ffi_desktop.dll`

### 2) Configure and build Qt app
From repository root:

```bash
cmake -S apps/desktop -B build/desktop -DFFI_LIB_DIR=$(pwd)/core/target/release
cmake --build build/desktop
```

### 3) Run
With file argument:

```bash
./build/desktop/craftcad_desktop /path/to/project.diycad
```

Or without argument to use file picker:

```bash
./build/desktop/craftcad_desktop
```

## Notes
- This app is intentionally read-only.
- Rendering currently includes `Line` only (other geometry types can be added later).

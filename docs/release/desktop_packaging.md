# Desktop packaging v1

## Windows
- Build with MSVC + Qt6
- Bundle `craftcad_desktop` executable and required Qt runtime DLLs
- Include `README` and license files

## macOS
- Build app bundle via CMake
- Ensure Qt frameworks are bundled and signed
- Export `.dmg` in release pipeline

## Diagnostic defaults
- Diagnostic pack excludes `.diycad` snapshot by default
- User must explicitly opt in to include project snapshot

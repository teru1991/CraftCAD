# Threat Model (CraftCAD / Sprint18 SSOT)

## Scope
This document defines the v1 security & privacy contract for untrusted inputs:
- Project package (.diycad, zip)
- Import formats (DXF / SVG / JSON)
- Presets / templates
- Diagnostics artifacts (JobLog / SupportZip)

## Assets to protect
- User privacy (PII, file paths, user names, tokens)
- Application availability (no crash / no hang on malformed or huge inputs)
- User files integrity (no path traversal / no arbitrary overwrite)

## Threats
- Zip bomb (huge decompressed size, too many entries)
- Huge JSON / deep nesting to trigger DoS or stack overflow
- Malicious SVG (external refs via href/xlink:href, external CSS)
- Corrupted DXF/SVG/JSON to trigger panic
- Path traversal: ../, absolute paths, device paths, UNC paths
- PII leakage through logs / diagnostics zip

## Mitigations
- Limits preflight for any untrusted input before parse/decompress
- Path normalization & strict validation for zip entry names and output paths
- External references are rejected or stripped per policy (SSOT-driven)
- Redaction of all diagnostics/log content and filenames
- Consent is opt-in and revocable; defaults are OFF; consent stored in user settings (not project)
- Atomic write for generated outputs; never overwrite outside allowed roots

## Out of scope (v1)
- Advanced sandboxing, code signing verification of content packs, encrypted storage
- Full SBOM signing pipeline (only SCA gate + exception workflow in v1)

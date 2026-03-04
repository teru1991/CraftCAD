# Consent Policy (Telemetry / Diagnostics)

## Defaults
- All data collection and sharing is OFF by default (opt-in).

## What may be collected / exported
- Telemetry: minimal product health metrics (if enabled)
- Diagnostics: SupportZip containing logs and redacted job metadata

## User controls
- Users can change or revoke consent at any time.
- SupportZip creation MUST show a choice:
  - Use current saved consent
  - Enable for this export only (one-time)
  - Cancel

## Privacy constraints (non-negotiable)
- No project content or raw input files are included unless explicitly enabled.
- All exported artifacts must be redacted (paths, emails, tokens, free text).

## Storage
- Consent is stored in user settings (device/local), not inside project files.

## Failure behavior
- If consent settings are missing or corrupted: reset to defaults (OFF) and emit a warning code.

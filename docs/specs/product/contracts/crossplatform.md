# Crossplatform Guard Contract

## Path / Encoding
- Path values crossing FFI boundaries must be handled as UTF-8 strings.
- Callers must not assume ASCII-only file names.

## Newline
- Repository text files use LF (`\n`).
- Scripts/checkers should tolerate CRLF input where needed, but generated outputs must remain LF.

## Executable scripts
- CI entrypoint scripts must be executable in git index mode (`100755`).
- `crossplatform_checks` verifies executable mode for critical scripts.

## Atomic save
- Save flow must use temp-file write in the same directory followed by rename/replace.
- On Windows rename collision/lock behavior, implementation should handle retry/backoff or replace fallback.

## CI guard
- `scripts/ci/crossplatform_checks.py` runs in CI (Linux runner) to detect common crossplatform pitfalls early.

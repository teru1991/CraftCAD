# G1-8-CROSSPLATFORM-GUARDS-001 Verification

## Summary
- Added a crossplatform contract document for path encoding, newline policy, executable script mode, atomic save expectations, and CI guard scope.
- Added `scripts/ci/crossplatform_checks.py` to detect common crossplatform regressions preemptively.
- Wired crossplatform checks into unified CI entrypoint (`scripts/ci/run_all.sh`) as an early gate.
- Added LF/unicode fixtures under `tests/fixtures/` and self-test mode in checker for CRLF detection logic.

## Changed files
- `docs/specs/product/contracts/crossplatform.md`
- `scripts/ci/crossplatform_checks.py`
- `scripts/ci/run_all.sh`
- `scripts/ci/trace_index_check.py`
- `tests/fixtures/crossplatform_lf_sample.txt`
- `tests/fixtures/crossplatform_unicode_パス.txt`
- `docs/verification/G1-8-CROSSPLATFORM-GUARDS-001.md`
- `docs/status/trace-index.json`

## History evidence
- `git status --porcelain` (before): clean.
- `git fetch --all --prune`: success.
- `git switch -c feature/g1-8-crossplatform-guards-001`: branch created.
- `git rev-parse HEAD`: `feff4401328d202193695627d1e8ce56c5a9576d`.
- `git log -n 40 --oneline`: reviewed recent history.
- `git branch -vv`: confirmed active branch.

## Existing assumptions evidence
- `shellcheck -V || true`: `shellcheck` not installed in this environment.
- `rg -n "sed -i|readlink -f|/usr/bin/env bash|chmod \+x|apt-get" scripts -S`: checked script patterns currently used.
- `rg -n "atomic|rename|tempfile|write\+rename|fs::rename" core/crates -S`: confirmed atomic-save related implementation signals in core.

## Local verification
- `python3 scripts/ci/crossplatform_checks.py --self-test`: pass.
- `./scripts/ci/run_all.sh`:
  - `crossplatform_checks` step passed.
  - existing unrelated failures remained (`rust_test`, `e2e_shelf_flow`).
  - desktop gate skip behavior remained explicit on Qt-unavailable environment.
- `python -m json.tool docs/status/trace-index.json >/dev/null`: pass.

## Self-check
- Allowlist respected (`docs/**`, `scripts/**`, `tests/**`).
- No deletions.
- `trace-index.json` remains valid JSON.

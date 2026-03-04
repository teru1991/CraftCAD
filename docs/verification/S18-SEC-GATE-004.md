# Verification: S18-SEC-GATE-004

## Goal
Add SCA CI gate (cargo-audit) with time-bounded allowlist, and finalize Sprint18 test gates (golden/malformed/E2E/smoke fuzz).

## Changed files
- .github/workflows/security.yml
- scripts/ci/sca_allowlist_check.py
- docs/specs/security/sca_allowlist.schema.json
- docs/specs/security/sca_allowlist.json
- tests/security/redaction_golden.rs
- tests/security/malformed_inputs.rs
- tests/e2e/support_zip_is_safe.rs
- tests/fuzz/security_smoke_fuzz.rs
- core/crates/ssot_lint/tests/{redaction_golden,malformed_inputs,support_zip_is_safe,security_smoke_fuzz}.rs
- core/crates/ssot_lint/Cargo.toml
- docs/status/trace-index.json

## History evidence
- `git log --oneline -n 30`:
  - `8788643 S18: enforce security integration across format/io/diagnostics/migrate`
  - `728fc40 S18: implement security core crate with SSOT-driven guards`
  - `4b66703 Merge pull request #92 from teru1991/codex/establish-ssot-for-security-specifications`
- `git log -- .github/workflows | head`:
  - `commit 395ae05d5b482607a32183e649f88430cc1b60cf`
  - `Author: teru1991 <48640151+teru1991@users.noreply.github.com>`
- `git log -- docs/specs/security/sca_policy.md | head`:
  - `commit 7e808d2352e4c90dfd64293a403331dc3b46c75e`
  - `Author: teru1991 <48640151+teru1991@users.noreply.github.com>`

## Commands executed
- `python3 scripts/ci/sca_allowlist_check.py`
- `cargo test --manifest-path core/Cargo.toml -p security`
- `cargo test --manifest-path core/Cargo.toml -p craftcad_io_svg --test svg_limits_external_ref`
- `cargo test --manifest-path core/Cargo.toml -p ssot_lint --test redaction_golden --test malformed_inputs --test support_zip_is_safe --test security_smoke_fuzz`
- `cargo fmt` (failed due existing unresolved conflict markers outside this task)

## Assertions
- SCA gate includes allowlist expiry validation and strict audit failure behavior.
- Allowlist exceptions are SSOT-managed and time-bounded by `expires_on`.
- Redaction golden, malformed-input guards, support-zip PII scan, and short fuzz smoke are executed in PR gate workflow.

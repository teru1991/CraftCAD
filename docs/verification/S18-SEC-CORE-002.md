# Verification: S18-SEC-CORE-002

## Goal
Implement `core/crates/security` (Limits/Sandbox/Redaction/Consent) with panic-free, deterministic behavior and SSOT loading.

## Changed files
- core/crates/security/Cargo.toml
- core/crates/security/src/lib.rs
- core/crates/security/src/reasons.rs
- core/crates/security/src/ssot.rs
- core/crates/security/src/util.rs
- core/crates/security/src/limits.rs
- core/crates/security/src/sandbox.rs
- core/crates/security/src/redaction.rs
- core/crates/security/src/consent.rs
- core/crates/diagnostics/Cargo.toml
- docs/status/trace-index.json

## Spec alignment (Sprint18)
- Limits: `Limits::load_from_ssot(profile)` + `check_bytes` / `check_zip` / `check_json_depth` with `SEC_LIMIT_*` and `SEC_ZIP_*` reason codes.
- Sandbox: `normalize_rel_path` and SVG external reference reject/strip handling with `SEC_PATH_*` and `SEC_EXTERNAL_REF_*` reason codes.
- Redaction: `redact_str` / `redact_path` / `redact_json` using SSOT rules, deterministic order, and free-text hashing.
- Consent: defaults OFF, deterministic load/save, and corruption reset warning (`SEC_CONSENT_RESET`).

## History evidence
- `git log --oneline -n 30`:
  - `4b66703 Merge pull request #92 from teru1991/codex/establish-ssot-for-security-specifications`
  - `7e808d2 S18: lock security SSOT and enforce lint checks`
  - `e5a7351 Merge pull request #91 from teru1991/codex/implement-diagnostics-retention-and-ssot-fingerprint-5rylv4`
- `git log -- docs/specs/security/limits.json | head`:
  - `commit 7e808d2352e4c90dfd64293a403331dc3b46c75e`
  - `Author: teru1991 <48640151+teru1991@users.noreply.github.com>`
  - `Date:   Wed Mar 4 11:26:58 2026 +0900`
  - `S18: lock security SSOT and enforce lint checks`
- `git log -- core/crates | head`:
  - `commit 2930a7f1588e35f624b23ce1d5ac7c4428c09d18`
  - `Merge branch 'main' into codex/implement-diagnostics-retention-and-ssot-fingerprint-5rylv4`

## Commands executed
- `git status --porcelain`
- `git fetch --all --prune`
- `git checkout -b feature/s18-sec-core-002`
- `scripts/ci/run_all.sh` (failed at rust_fmt due existing merge conflict markers outside this change)
- `rustfmt core/crates/security/src/*.rs`
- `cargo test -p security`
- `cargo clippy -p security -- -D warnings`

## Safety checks
- Allowed-paths only: yes
- No deletions: yes
- No panics introduced in runtime paths: yes (`SecError`/`SecResult` based failures)
- Determinism: yes (redaction unit test validates same input => same output)

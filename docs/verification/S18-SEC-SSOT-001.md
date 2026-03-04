# Verification: S18-SEC-SSOT-001

## Goal
Establish security SSOT (limits/redaction/consent/sca policy) and enforce via spec SSOT lint.

## Changed files
- docs/specs/security/threat_model.md
- docs/specs/security/limits.schema.json
- docs/specs/security/limits.json
- docs/specs/security/redaction.schema.json
- docs/specs/security/redaction_rules.json
- docs/specs/security/consent.md
- docs/specs/security/consent.schema.json
- docs/specs/security/sca_policy.md
- docs/specs/security/README.md
- core/serialize/tests/spec_ssot_lint.rs
- docs/status/trace-index.json
- docs/verification/S18-SEC-SSOT-001.md

## SSOT alignment (B19-01..04)
- B19-01: limits.json + threat_model mitigations lock preflight limits and malformed-input resilience.
- B19-02: redaction_rules.json + lint(regex compile) lock deterministic PII redaction contract.
- B19-03: consent.md/schema + lint(required phrases) lock opt-in/revocable/one-time consent and user-settings storage.
- B19-04: sca_policy.md documents release-blocking HIGH/CRITICAL gate and time-bounded exception policy.

## History evidence
- git log --oneline -n 20:
  - e5a7351 Merge pull request #91 from teru1991/codex/implement-diagnostics-retention-and-ssot-fingerprint-5rylv4
  - 2930a7f Merge branch 'main' into codex/implement-diagnostics-retention-and-ssot-fingerprint-5rylv4
  - 61dfcc4 S17: add diagnostics quality gates (golden/determinism/e2e/fuzz)
  - a2e52ae Merge pull request #90 from teru1991/codex/implement-diagnostics-retention-and-ssot-fingerprint-1tt9p1
  - 953531c Merge branch 'main' into codex/implement-diagnostics-retention-and-ssot-fingerprint-1tt9p1
  - fd25a1d S17: add diagnostics CLI support workflow entrypoints
  - 24f4eab Merge pull request #89 from teru1991/codex/implement-diagnostics-retention-and-ssot-fingerprint
  - a5a296e S17: implement diagnostics retention store and SSOT fingerprint
  - f99174e Merge pull request #87 from teru1991/codex/add-diagnostics-ssot-files-and-lint-yjw5g3
  - d39582c Merge branch 'main' into codex/add-diagnostics-ssot-files-and-lint-yjw5g3
  - 2b9b8de S17: implement diagnostics core crate with deterministic job/op logs
  - ac5c1d3 Merge pull request #86 from teru1991/codex/add-diagnostics-ssot-files-and-lint
  - 691aed7 S17: add diagnostics SSOT specs and lint gate
  - 3946aec Merge pull request #85 from teru1991/codex/implement-ssot-contracts-for-performance-e4vjxw
  - 5f15c15 Merge branch 'main' into codex/implement-ssot-contracts-for-performance-e4vjxw
  - cb7e1ac S16: integrate perf gate policy and reproducible artifact collection
  - c74aa37 Merge pull request #84 from teru1991/codex/implement-ssot-contracts-for-performance-0gqdms
  - 7174be0 Merge branch 'main' into codex/implement-ssot-contracts-for-performance-0gqdms
  - 6afcf25 S16: add perf bench executables and report artifact tooling
  - 81bed05 Merge pull request #83 from teru1991/codex/implement-ssot-contracts-for-performance-ndqnlo
- git log -- core/serialize/tests/spec_ssot_lint.rs | head:
  - 691aed7 S17: add diagnostics SSOT specs and lint gate
  - cb7e1ac S16: integrate perf gate policy and reproducible artifact collection
  - e0c300a Sprint13: integrate presets/templates references into .diycad document with migration (step6)
  - b593228 Sprint13: add presets/templates/library SSOT and lint gate (step1)
  - eaeecda 追加、修正
  - 00a064e Add mapping rules doc target required by SSOT lint
- git blame core/serialize/tests/spec_ssot_lint.rs (top) summary:
  - Initial SSOT lint skeleton authored mainly by teru1991 (79e50ce5, d3345437).
  - Early-line maintenance includes follow-up fixes from 59bca099.

## Commands executed
- `git status --porcelain` → clean
- `git fetch --all --prune` → success
- `git checkout -b feature/s18-sec-ssot-001` → success
- `git log --oneline -n 20` → captured above
- `git log -- core/serialize/tests/spec_ssot_lint.rs | head -n 40` → captured above
- `git blame core/serialize/tests/spec_ssot_lint.rs | head -n 50` → captured summary above
- `cargo fmt --all` (repo root) → failed (no root Cargo.toml)
- `cargo fmt --manifest-path core/Cargo.toml --all` → failed due to pre-existing conflict marker in `core/crates/ssot_lint/tests/golden_datasets.rs`
- `rustfmt core/serialize/tests/spec_ssot_lint.rs` → success
- `cargo clippy --manifest-path core/serialize/Cargo.toml --all-targets -- -D warnings` → success
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` → success (includes `spec_ssot_lint` with `ssot_security_contracts_exist_and_valid` pass)
- `scripts/ci/run_all.sh` → partial failure at existing diagnostics jobs (`diagnostics_tests`, `diagnostics_golden`, `diagnostics_support_zip_e2e`)

## Safety checks
- Allowed-paths only: yes
- No deletions: yes
- Panic-free lint: yes for SSOT security checks (invalid/missing structure reported with deterministic assertion messages; regex patterns are compiled and validated during lint)

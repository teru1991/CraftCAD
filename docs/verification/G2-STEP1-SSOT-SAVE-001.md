# Verification: G2-STEP1-SSOT-SAVE-001

## Summary
This task fixes SSOT save/load contract at project-file layer by introducing `ssot_v1` persistence and backward-compatible derivation.
- Added SSOT v1 model types (`Part/Material/FeatureGraph`) with `ssot_version=1` and canonicalization.
- Wired `.diycad` save/load to persist optional `ssot_v1.json` and derive deterministic minimal SSOT when missing.
- Added tests for deterministic UUID generation, SSOT roundtrip persistence, and legacy-load derivation.

## Changed files
- docs/specs/product/contracts/master-model.md
- docs/specs/product/contracts/derived-artifacts.md
- core/Cargo.toml
- core/Cargo.lock
- core/crates/craftcad_ssot/Cargo.toml
- core/crates/craftcad_ssot/src/lib.rs
- core/crates/craftcad_ssot/tests/deterministic_uuid.rs
- core/crates/diycad_project/Cargo.toml
- core/crates/diycad_project/src/lib.rs
- docs/status/trace-index.json (tasks["G2-STEP1-SSOT-SAVE-001"] only)

## History evidence
### Preflight
- `git status --porcelain`:

```text
<empty>
```

- `git rev-parse HEAD`:

```text
8d635bbc2d2388e22efad0d08f2913ca6faa8844
```

- `git log -n 30 --oneline`:

```text
8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
17e0feb docs(product): add implementation readiness SSOT for final pillars (P00-PREP-SSOT-001)
...
```

- `git branch -vv`:

```text
* feature/g2-step1-ssot-save-001     8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
  feature/g1-step1-desktop-build-001 8d635bb build(desktop): unify release build route and CI to use scripts/build_desktop.sh (G1-STEP1-DESKTOP-BUILD-001)
  work                               eb110ea Merge pull request #104 from teru1991/codex/add-implementation-readiness-ssot
```

### Schema/load-save entrypoint findings
Executed scans:
- `rg -n "project_file|ProjectFile|diycad_project|save\(|load\(|serde\(rename|schema_version" core/crates -S`
- `rg -n "\.diycad|support_zip|migration" core/crates -S`
- `rg -n "uuid::Uuid|Uuid" core/crates -S`

Chosen files and rationale:
- `core/crates/diycad_project/src/lib.rs`: owns `.diycad` ZIP save/load (`save`, `load`, `manifest.json`, `data.json`) so this is the correct integration point for optional SSOT persistence and legacy derivation.
- `core/crates/diycad_project/Cargo.toml`: needed to link SSOT types crate.
- `core/Cargo.toml`: workspace member registration for new SSOT crate.

## Local verification
- `cargo test -p craftcad_ssot`

Result:

```text
running 2 tests
... ok
```

- `cargo test -p diycad_project`

Result:

```text
running 6 tests
... ok
```

- Wiring check:

```bash
rg -n "ssot_v1|derive_ssot_from_legacy|ssot_version|deterministic_uuid" core/crates -S
```

Result summary:

```text
core/crates/diycad_project/src/lib.rs includes Optional ssot_v1 field, save/write of ssot_v1.json,
load-time optional parse + derive_ssot_from_legacy fallback, and tests for roundtrip + legacy derivation.
core/crates/craftcad_ssot/src/lib.rs includes ssot_version, deterministic_uuid (SHA-256 based), and derive_minimal_ssot_v1.
```

## Self-check
- Allowlist respected: YES
- No deletions: YES
- json.tool trace-index: PASS

## Notes
- `ssot_v1` remains optional for backward compatibility.
- On missing SSOT, derivation is deterministic (stable ordering + deterministic UUIDs) and becomes persisted on next save.

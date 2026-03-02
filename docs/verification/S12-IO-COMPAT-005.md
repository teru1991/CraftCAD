# Verification: S12-IO-COMPAT-005

## Goal
Sprint12(I/O拡張完成)の最終ゲート：DXF/SVG/JSON round-trip互換を自動検証（golden + compat report）し、SSOT逸脱をCIで即検知する。決定性・互換・失敗説明（ReasonCode）を“製品品質”として固定する。

## Spec / SSOT alignment
- `docs/specs/io/compat_policy.md` を新設し、許容差分/不許容差分/レポートschema(v1)を固定
- `docs/specs/io/format_policy.md` から compat_policy を参照
- SSOTゲート:
  - support_matrix reason_codes は errors/catalog.json に存在すること
  - support_matrix の (format,direction,feature) が一意であること
  - mapping_rules の schema_version/regex/units/export の不変条件を検証

## Changed Files
- docs/specs/io/compat_policy.md
- docs/specs/io/format_policy.md
- core/crates/io_support/tests/io_contract_lint.rs
- core/crates/io_bridge/tests/ssot_gate.rs
- core/crates/io_bridge/tests/compat_matrix_golden.rs
- core/crates/io_bridge/Cargo.toml
- tests/golden/io_roundtrip/inputs/compat/compat_01.json
- tests/golden/io_roundtrip/expected/compat/compat_report.json
- scripts/ci/run_io_golden.sh
- docs/status/trace-index.json
- docs/verification/S12-IO-COMPAT-005.md

## Preflight & History Evidence (paste outputs)
- `git status`
```text
On branch feature/s12-io-compat-005-001
Changes not staged for commit:
  modified:   core/crates/io_bridge/Cargo.toml
  modified:   core/crates/io_support/tests/io_contract_lint.rs
  modified:   docs/specs/io/format_policy.md
Untracked files:
  core/crates/io_bridge/tests/compat_matrix_golden.rs
  core/crates/io_bridge/tests/ssot_gate.rs
  docs/specs/io/compat_policy.md
  scripts/ci/run_io_golden.sh
  tests/golden/io_roundtrip/expected/compat/
  tests/golden/io_roundtrip/inputs/compat/
```
- `git branch -vv`
```text
* feature/s12-io-compat-005-001 f0c446e S12-IO-BRIDGE-004: unify shared IO pipeline and E2E bridge goldens
  feature/s12-io-bridge-004-001 f0c446e S12-IO-BRIDGE-004: unify shared IO pipeline and E2E bridge goldens
  work                          e679b20 Productize DXF/SVG IO: SSOT mapping, DXF header/units, polyline bulge/arc, SVG path & transform parsing, and tests
```
- `git log --oneline --decorate -n 30` / `git log --oneline --decorate --graph --max-count=100`
```text
HEAD: f0c446e (feature/s12-io-compat-005-001) S12-IO-BRIDGE-004: unify shared IO pipeline and E2E bridge goldens
prev: e679b20 Productize DXF/SVG IO: SSOT mapping, DXF header/units, polyline bulge/arc, SVG path & transform parsing, and tests
history includes S12 SSOT/SVG/DXF/BRIDGE integration chain and merges through #56.
```
- `git show --stat`
```text
f0c446e S12-IO-BRIDGE-004: unify shared IO pipeline and E2E bridge goldens
 15 files changed, 867 insertions(+), 127 deletions(-)
```
- `git blame docs/specs/io/support_matrix.json | head -n 200`
- `git blame docs/specs/io/mapping_rules.json | head -n 200`
- `git blame docs/specs/errors/catalog.json | head -n 120`
- `git blame core/crates/io_bridge/tests/e2e_roundtrip_matrix.rs | head -n 240`
```text
Executed successfully to confirm ownership/history for SSOT and bridge e2e harness.
```

## Tests Run (commands + results)
- `GOLDEN_ACCEPT=1 scripts/ci/run_io_golden.sh` ✅
- `scripts/ci/run_io_golden.sh` ✅
- `cd core && cargo test -p craftcad_io_bridge --tests` ✅
- `cd core && cargo test -p craftcad_io_support --tests` ✅

## Self-check (Hard rules)
- [x] Only allowed paths touched
- [x] No deletions (only edits/additions)
- [x] Spec updated first (compat_policy/format_policy)
- [x] Added tests (ssot gate + compat golden)
- [x] Determinism preserved (same input -> same outputs)
- [x] SSOT drift detected (unknown reason_codes/duplicate matrix keys)
- [x] Linux-only script works (no pwsh)

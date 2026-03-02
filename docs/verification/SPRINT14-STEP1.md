# Verification: SPRINT14-STEP1

## Goal
- Sprint14 Step1: SSOT(.diycad)完成 + SSOT Lint導入

## Scope / Changed files
- docs/specs/schema/diycad/{format.md,versions.md,migration_policy.md,integrity.md,recovery.md}
- docs/specs/schema/diycad/{manifest.schema.json,document.schema.json,part.schema.json,nest_job.schema.json}
- core/crates/ssot_lint/** (new)
- scripts/ci/run_all.sh
- core/Cargo.toml / core/Cargo.lock
- docs/status/trace-index.json

## SSOT alignment
- DBなし、正本は.diycad。アプリは設定/キャッシュのみ（SSOT準拠）
- 互換: N-2 読み込み / 最新のみ書き出し / 段階migration（SSOT準拠）
- 落ちない/救える/説明できる（integrity.md/recovery.mdで契約化）

## History evidence (paste outputs)
- git log -n 20 --oneline:
  - f239d3d Merge pull request #67 from teru1991/codex/add-quality-gates-for-sprint-13
  - 395ae05 Sprint13: add quality gates (compat/determinism/e2e) for presets+wizards (step7)
  - 1c41875 Merge pull request #66 from teru1991/codex/implement-library-crate-with-unit-tests-rjmc32
  - e81feb0 Merge branch 'main' into codex/implement-library-crate-with-unit-tests-rjmc32
  - e0c300a Sprint13: integrate presets/templates references into .diycad document with migration (step6)
  - 40bf586 Merge pull request #65 from teru1991/codex/implement-library-crate-with-unit-tests-8atvfm
  - d3b1472 Merge branch 'main' into codex/implement-library-crate-with-unit-tests-8atvfm
  - 05c0443 Sprint13: implement shelf/box/leather wizards generating PartsDraft (step5)
  - f862a1d Merge pull request #64 from teru1991/codex/implement-library-crate-with-unit-tests-i0tgs1
  - d2e6029 Sprint13: add wizards template engine with safe DSL and determinism (step4)
  - 07658d4 Merge pull request #63 from teru1991/codex/implement-library-crate-with-unit-tests
  - 663121b Sprint13: add library crate with tags/index/deps/store (step3)
  - ba15546 Merge pull request #62 from teru1991/codex/complete-ssot-for-presets-and-templates-n1y82g
  - df9fee3 Merge branch 'main' into codex/complete-ssot-for-presets-and-templates-n1y82g
  - e915fbb Sprint13: add wizards template engine with safe DSL and determinism (step4)
  - 05c3594 Merge pull request #61 from teru1991/codex/complete-ssot-for-presets-and-templates
  - b593228 Sprint13: add presets/templates/library SSOT and lint gate (step1)
  - ed95e78 Merge pull request #60 from teru1991/codex/improve-svg-import-quality-1rijgm
  - 8923b6a Merge branch 'main' into codex/improve-svg-import-quality-1rijgm
  - e0e8b27 S12-IO-COMPAT-005: add compat policy, SSOT gates, and golden runner
- git branch -vv:
  - * feature/sprint14-step1-001 f239d3d Merge pull request #67 from teru1991/codex/add-quality-gates-for-sprint-13
  -   work                       f239d3d Merge pull request #67 from teru1991/codex/add-quality-gates-for-sprint-13
- git rev-parse HEAD:
  - f239d3d5bf043ff04dbe58df2bfbdd9a540d3a46

## Commands executed
- cargo test -q (repo root):
  - error: could not find `Cargo.toml` in `/workspace/CraftCAD` or any parent directory
- cargo test -p ssot_lint (core):
  - test result: ok. 1 passed; 0 failed
- cargo run -p ssot_lint --bin ssot-lint (core):
  - ssot-lint: OK
- scripts/ci/run_all.sh:
  - rust_fmt: PASS
  - rust_clippy: FAIL (existing workspace issues)
  - rust_test: FAIL (existing workspace issues)
  - ssot_lint: PASS
  - e2e_shelf_flow: PASS
  - determinism_wizard: PASS
  - compat_presets_templates: PASS
  - rust_ffi_desktop: PASS
  - rust_ffi_build: PASS
  - cmake_configure: FAIL
  - cmake_build: FAIL

## Notes
- This step intentionally does not implement runtime open/save/recovery/migrate code.
- It establishes SSOT + gate so later steps can be safely merged.

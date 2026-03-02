# Verification: SPRINT14-STEP2

## Goal
- diycad_format の open/save/integrity/limits を販売品質で実装

## Changed files
- core/crates/diycad_format/** (open/save/package/limits/integrity/types/reasons + tests)
- scripts/ci/run_all.sh
- core/Cargo.lock
- docs/status/trace-index.json

## History evidence
- git log -n 20 --oneline:
  - 7a3c97e Sprint14 Step1: Add DIYCAD SSOT docs and ssot-lint gate
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
- git branch -vv:
  - * feature/sprint14-step2-001 7a3c97e Sprint14 Step1: Add DIYCAD SSOT docs and ssot-lint gate
  -   work                       7a3c97e Sprint14 Step1: Add DIYCAD SSOT docs and ssot-lint gate
- git rev-parse HEAD:
  - 7a3c97e3bdb8b628f6dd35278d11a01999f53fd9

## Commands executed (paste outputs)
- cargo test -p diycad_format:
  - test result: ok. 4 passed; 0 failed
- cargo test -p diycad_format --tests:
  - test result: ok. 4 passed; 0 failed
- scripts/ci/run_all.sh:
  - rust_fmt: PASS
  - rust_clippy: FAIL (existing workspace issues)
  - rust_test: FAIL (existing workspace issues)
  - diycad_format_tests: PASS
  - ssot_lint: PASS
  - e2e_shelf_flow: PASS
  - determinism_wizard: PASS
  - compat_presets_templates: PASS
  - rust_ffi_desktop: PASS
  - rust_ffi_build: PASS
  - cmake_configure: FAIL
  - cmake_build: FAIL

## Key guarantees
- Determinism: warnings/failed orders stable; open repeated runs identical signature (open_determinism.rs)
- Limits: too-many-entries rejects without crash (limits_zipbomb.rs)
- Atomic save: target remains; temp not left behind (atomic_save.rs)

## Notes / follow-ups
- JSON Schema（manifest/document/part/nest_job）の厳密検証は Step3/Step4 で schema validator を導入して強化する。
- Migration crate 連携は Step4/Step5 で実装する（このStepでは hook まで）。

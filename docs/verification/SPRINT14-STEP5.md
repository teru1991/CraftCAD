# Verification: SPRINT14-STEP5

## Goal
- recovery crate (autosave/restore) + crash recovery E2E

## Changed files
- core/crates/recovery/** (new)
- core/Cargo.toml
- core/crates/wizards/Cargo.toml
- tests/e2e/project_crash_recovery.rs
- scripts/ci/run_all.sh
- docs/status/trace-index.json

## History evidence (paste)
- git log -n 20 --oneline:
  - 18144f3 Merge pull request #70 from teru1991/codex/complete-migration-crate-for-stepwise-migration
  - c7e6bfd Sprint14 Step4: add stepwise migration engine and diycad hook
  - a266e5d Merge pull request #69 from teru1991/codex/complete-sprint14-step1-with-ssot-lint-dwloto
  - 653acc4 Merge branch 'main' into codex/complete-sprint14-step1-with-ssot-lint-dwloto
  - 8675146 Merge pull request #68 from teru1991/codex/complete-sprint14-step1-with-ssot-lint
  - ce822ff Sprint14 Step2: implement resilient diycad_format open/save pipeline
  - 9b6dac6 Sprint14 Step1: finalize DIYCAD SSOT and add ssot lint gate
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
- git branch -vv:
  - * feature/sprint14-step5-001 18144f3 Merge pull request #70 from teru1991/codex/complete-migration-crate-for-stepwise-migration
  - work 18144f3 Merge pull request #70 from teru1991/codex/complete-migration-crate-for-stepwise-migration
- git rev-parse HEAD:
  - 18144f379e41084bd68bfd6690d2bad865d79a87

## Commands executed (paste)
- cargo test -p recovery (pass)
- cargo test -p craftcad_wizards --test project_crash_recovery (pass)
- scripts/ci/run_all.sh (fails at pre-existing rust_fmt/rust_clippy/rust_test/cmake steps; new recovery and crash-recovery gates pass)

## Acceptance
- dirty時のみautosaveされる
- atomic保存で世代が壊れない
- 世代ローテ/容量上限が決定的に働く
- *.tmp（保存中クラッシュ残骸）があっても復旧できる

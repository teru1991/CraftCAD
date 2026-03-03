# Verification: SPRINT14-STEP8

## Goal
- Sprint14 「データが死なない」最終統合確認 + runbook/trace-index 仕上げ。

## Changed files
- docs/runbooks/sprint14_project_survival.md
- scripts/ci/run_all.sh
- docs/status/trace-index.json
- docs/verification/SPRINT14-STEP8.md

## History evidence (paste)
- git log -n 30 --oneline:
  ```text
  87027ac Merge pull request #73 from teru1991/codex/add-pr-gated-testing-framework
  acac9d2 test: add PR-gated golden/compat/fuzz/determinism/e2e (binary-free)
  94ae6f9 Merge pull request #72 from teru1991/codex/complete-tools/migrate-for-cli-safety-features
  f7c5f4a Sprint14 Step6: complete diycad-migrate CLI with deterministic batch summary
  50f3cbb Merge pull request #71 from teru1991/codex/implement-recovery-crate-with-autosave
  1853b7f Sprint14 Step5: add recovery autosave/restore and crash recovery E2E
  18144f3 Merge pull request #70 from teru1991/codex/complete-migration-crate-for-stepwise-migration
  c7e6bfd Sprint14 Step4: add stepwise migration engine and diycad hook
  a266e5d Merge pull request #69 from teru1991/codex/complete-sprint14-step1-with-ssot-lint-dwloto
  653acc4 Merge branch 'main' into codex/complete-sprint14-step1-with-ssot-lint-dwloto
  8675146 Merge pull request #68 from teru1991/codex/complete-sprint14-step1-with-ssot-lint
  ce822ff Sprint14 Step2: implement resilient diycad_format open/save pipeline
  9b6dac6 Sprint14 Step1: finalize DIYCAD SSOT and add ssot lint gate
  f239d3d Merge pull request #67 from teru1991/codex/add-quality-gates-for-sprint-13
  395ae05 Sprint13: add quality gates (compat/determinism/e2e) for presets+wizards (step7)
  1c41875 Merge pull request #66 from teru1991/codex/implement-library-crate-with-unit-tests-rjmc32
  e81feb0 Merge branch 'main' into codex/implement-library-crate-with-unit-tests-rjmc32
  e0c300a Sprint13: integrate presets/templates references into .diycad document with migration (step6)
  40bf586 Merge pull request #65 from teru1991/codex/implement-library-crate-with-unit-tests-8atvfm
  d3b1472 Merge branch 'main' into codex/implement-library-crate-with-unit-tests-8atvfm
  05c0443 Sprint13: implement shelf/box/leather wizards generating PartsDraft (step5)
  f862a1d Merge pull request #64 from teru1991/codex/implement-library-crate-with-unit-tests-i0tgs1
  d2e6029 Sprint13: add wizards template engine with safe DSL and determinism (step4)
  07658d4 Merge pull request #63 from teru1991/codex/implement-library-crate-with-unit-tests
  663121b Sprint13: add library crate with tags/index/deps/store (step3)
  ba15546 Merge pull request #62 from teru1991/codex/complete-ssot-for-presets-and-templates-n1y82g
  df9fee3 Merge branch 'main' into codex/complete-ssot-for-presets-and-templates-n1y82g
  e915fbb Sprint13: add wizards template engine with safe DSL and determinism (step4)
  05c3594 Merge pull request #61 from teru1991/codex/complete-ssot-for-presets-and-templates
  b593228 Sprint13: add presets/templates/library SSOT and lint gate (step1)
  ```
- git branch -vv:
  ```text
  * feature/sprint14-step8-001 87027ac Merge pull request #73 from teru1991/codex/add-pr-gated-testing-framework
    work                       87027ac Merge pull request #73 from teru1991/codex/add-pr-gated-testing-framework
  ```
- git rev-parse HEAD:
  ```text
  87027acb87596a7ebeda460cc02e32a959fa570f
  ```

## Commands executed (paste)
- scripts/ci/run_all.sh:
  ```text
  ==> rust_fmt
  [PASS] rust_fmt
  ==> rust_clippy
  [PASS] rust_clippy
  ==> rust_test
  [PASS] rust_test
  ==> diycad_format_tests
  [PASS] diycad_format_tests
  ==> diycad_format_tests_latest2
  [PASS] diycad_format_tests_latest2
  ==> migration_tests
  [PASS] migration_tests
  ==> ssot_lint
  [PASS] ssot_lint
  ==> e2e_shelf_flow
  [PASS] e2e_shelf_flow
  ==> determinism_wizard
  [PASS] determinism_wizard
  ==> compat_presets_templates
  [PASS] compat_presets_templates
  ==> golden_diycad_open_save
  [PASS] golden_diycad_open_save
  ==> compat_open
  [PASS] compat_open
  ==> fuzz_diycad_open_short
  [PASS] fuzz_diycad_open_short
  ==> determinism_open_signature
  [PASS] determinism_open_signature
  ==> e2e_migrate_verify_batch
  [PASS] e2e_migrate_verify_batch
  ==> recovery_tests
  [PASS] recovery_tests
  ==> e2e_crash_recovery
  [PASS] e2e_crash_recovery
  ==> tools_migrate_tests
  [PASS] tools_migrate_tests
  ```

## Final DoD checklist (all YES)
- [x] Atomic save: 旧ファイルを破壊しない（temp->fsync->rename）
- [x] Autosave recovery: クラッシュ後でも最後の良い世代へ戻れる（E2Eで保証）
- [x] Salvage: 破損/欠落でも落ちず read_only + FailedEntry + ReasonCode + salvage_actions を返す
- [x] Compatibility: N-2 読み込み方針がSSOTにあり、移行は段階的（engine+hooks）
- [x] Migration tool: dry-run/verify/diff/batch が安全に運用できる（in-placeなし）
- [x] Limits: zip bomb/巨大/深い/長い入力でクラッシュしない（tests+limits）
- [x] Determinism: 同一入力でOpenResult signatureが揺れない（test）
- [x] PR gate: ssot-lint + 全テストがCIで必ず回る（run_all.sh）

## Notes
- Sprint14完了としてマージ可能な状態: 上記DoDが全てYES、`scripts/ci/run_all.sh` 成功ログを本書に添付済み。

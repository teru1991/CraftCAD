# Verification: SPRINT14-STEP4

## Goal
- Stepwise migration engine + dry-run report + diycad_format hook.

## Changed files
- core/crates/migration/** (registry/step/report/reasons + tests)
- core/crates/diycad_format/** (migration dep, open integration, migrate_steps, types/reasons, tests)
- tools/migrate/Cargo.toml
- scripts/ci/run_all.sh

## History evidence
- git log -n 20 --oneline:
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
  - df9fee3 Merge branch 'main' into codex/complete-ssot-for-presets-and-templates-n1y82g
  - e915fbb Sprint13: add wizards template engine with safe DSL and determinism (step4)
- git branch -vv:
  - * feature/sprint14-step4-001 a266e5d Merge pull request #69 from teru1991/codex/complete-sprint14-step1-with-ssot-lint-dwloto
  -   work                       a266e5d Merge pull request #69 from teru1991/codex/complete-sprint14-step1-with-ssot-lint-dwloto
- git rev-parse HEAD:
  - a266e5d3b7090d96fb98aad8548f0db3ae7d3384

## Commands executed
- cargo test -p migration
- cargo test -p diycad_format
- cargo test -p diycad_format --features test_latest_2
- scripts/ci/run_all.sh (timeout 120)

## Acceptance
- migration registry enforces vN->vN+1 only and outputs deterministic ordered pointer sets via `BTreeSet`.
- diycad_format invokes migration hook when `schema_version < latest` and attaches `migrate_report` in `test_latest_2` mode.
- migration failure path degrades to read-only and salvage suggestion while continuing open flow.

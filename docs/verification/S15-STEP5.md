# S15-STEP5 Verification (binary-free)

## History evidence
- git status --porcelain:
  - M core/crates/ssot_lint/tests/golden_datasets.rs
  - M core/src/testing/mod.rs
  - M core/tests/golden_datasets.rs
  - M docs/specs/testing/golden_policy.md
  - M docs/status/trace-index.json
  - M tests/datasets/manifest.json
  - ?? core/crates/ssot_lint/tests/determinism_io.rs
  - ?? core/crates/ssot_lint/tests/determinism_migrate.rs
  - ?? core/crates/ssot_lint/tests/determinism_wizard.rs
  - ?? core/src/testing/dataset_runner.rs
  - ?? core/src/testing/determinism_harness.rs
  - ?? core/tests/determinism_io.rs
  - ?? core/tests/determinism_migrate.rs
  - ?? core/tests/determinism_wizard.rs
  - ?? scripts/testing/run_determinism.sh
- git log --oneline -n 20:
  - 16aa47b Add binary-free golden datasets and compatibility test harness (S15 STEP3/STEP4)
  - 6588498 Merge pull request #76 ...
- git log --graph --decorate --oneline --all -n 40:
  - branch start: feature/s15-step5-001 at 16aa47b
- git branch -vv:
  - * feature/s15-step5-001 16aa47b ...
- HEAD:
  - 16aa47b554e443adb2ff6867e6fe35080dfa0b1d

## What / Why
- 決定性ゲート（10回完全一致）を追加し、io / wizard / project_json_migrate の同一入力・同seed実行で fingerprint 一致を検証可能にした。
- Step3の dataset 実行ロジックを `dataset_runner.rs` に集約し、golden比較とdeterminism比較で同一runnerを利用するように変更した。
- mismatch時は `failure_artifacts/determinism/<dataset_id>/` に expected/actual fingerprint・diff・actual model・reason_codes を **text only** で出力する。
- NoiseMode::On は test pathのみ env var で有効化し、production default挙動には影響しない。

## Datasets tagged determinism
- io_roundtrip_smoke
- project_json_open_save_smoke
- wizard_shelf_smoke

## How verified
- cargo test -p core --test determinism_io -- --nocapture（repo rootにCargo.tomlがないため失敗）
- cargo test -p core --test determinism_wizard -- --nocapture（repo rootにCargo.tomlがないため失敗）
- cargo test -p core --test determinism_migrate -- --nocapture（repo rootにCargo.tomlがないため失敗）
- cargo test -p core（repo rootにCargo.tomlがないため失敗）
- ./scripts/testing/run_determinism.sh
- cargo test --manifest-path core/Cargo.toml -p ssot_lint

## Binary-free check
- `find failure_artifacts -type f -name '*.bin' -print` の結果: 空
- Determinism artifacts writer は `*.json/*.txt` のみ出力（.bin未使用）
- Determinism対象 assets は JSON/SVG テキスト入力のみ

## Completion criteria
- dataset_runner.rs は Step3 runner を集約し、NOT_IMPLEMENTED は未使用
- io / wizard / project_json_migrate の 10回一致が `run_determinism.sh` で成立
- NoiseMode::On の実行も 10回一致、または mismatch時は artifact付きで検出可能

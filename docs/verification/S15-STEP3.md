# S15-STEP3 Verification (binary-free)

## History evidence
- git status --porcelain:
  - M core/crates/ssot_lint/Cargo.toml
  - M core/src/testing/datasets_manifest.rs
  - M docs/status/trace-index.json
  - M tests/datasets/manifest.json
  - M tests/golden/README.md
  - M tests/golden/expected/io_roundtrip_smoke/normalized_model.json
  - ?? core/crates/ssot_lint/tests/golden_datasets.rs
  - ?? core/tests/golden_datasets.rs
  - ?? scripts/testing/
  - ?? tests/golden/expected/io_export_reimport_smoke/
  - ?? tests/golden/expected/project_json_open_save_smoke/
  - ?? tests/golden/expected/wizard_shelf_smoke/
  - ?? tests/golden/inputs/io/svg/rect_with_hole.svg
  - ?? tests/golden/inputs/projects/min_project_v1.json
  - ?? tests/golden/inputs/wizard/
- git log --oneline -n 20: captured locally (HEAD at `6588498` before this task work).
- git log --graph --decorate --oneline --all -n 40: captured locally (branch `work`).
- git branch -vv:
  - `* work 6588498 Merge pull request #76 ...`
- HEAD:
  - `65884987c5c0aab8ec067127f9deb509b814fb04`

## What / Why
- Golden “最小セット（バイナリ不使用版）” の入力資産を追加し、manifest SSOT に4 dataset を定義した。
- `diycad_open_save_smoke` は Step3 ポリシーに従い、`project_json_open_save_smoke`（text JSON）へ置換した。
- `saved_project` expected kind を追加し、golden dataset 比較テストで JSON/SVG/reason code を比較できるようにした。
- mismatch 時の差分は golden harness により `failure_artifacts/<dataset_id>/` へ出力される（実地で mismatch を1回発生→確認後、期待値を修正）。

## Golden update command (LOCAL ONLY)
- `./scripts/testing/run_golden.sh --accept --tags smoke`

## Datasets touched
- io_roundtrip_smoke
- io_export_reimport_smoke
- project_json_open_save_smoke
- wizard_shelf_smoke

## Binary-free check
- manifest参照の `tests/golden/inputs/**` / `tests/golden/expected/**` について以下を実行:
  - `./scripts/testing/run_golden.sh --tags smoke`
  - script 内チェック結果: `text-only fixture check: ok`

## How verified
- `./scripts/testing/run_golden.sh --accept --tags smoke` （golden_update 実行 + text-only check + compare）
- `./scripts/testing/run_golden.sh --tags smoke`
- `cargo test --manifest-path core/Cargo.toml -p ssot_lint --test golden_datasets -- --nocapture`
- `cargo test --manifest-path core/Cargo.toml -p ssot_lint`
- `cargo test -p core --test golden_datasets` （repo rootにCargo.tomlが無いため失敗）
- `cargo test -p core` （repo rootにCargo.tomlが無いため失敗）

## Notes / Artifacts
- mismatch確認例:
  - dataset: `io_export_reimport_smoke`
  - artifact path: `failure_artifacts/io_export_reimport_smoke/{meta.json,actual.json,diff.txt}`
  - expected `reimport_svg_len` を 259→283 に更新後、再実行で pass。

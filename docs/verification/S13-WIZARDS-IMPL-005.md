# Verification: S13-WIZARDS-IMPL-005

## Goal
3ウィザード（棚板/箱/革小物）をテンプレDSLからPartsDraftへ生成できるようにし、式評価/決定性/入力安全/回帰(golden)で品質ゲートする。

## Changed files
- `core/crates/wizards/src/lib.rs`
- `core/crates/wizards/src/shelf.rs`
- `core/crates/wizards/src/box.rs`
- `core/crates/wizards/src/leather_pouch.rs`
- `core/crates/wizards/src/parts/mod.rs`
- `core/crates/wizards/src/parts/model.rs`
- `core/crates/wizards/src/parts/validate.rs`
- `core/crates/wizards/src/parts/normalize.rs`
- `core/crates/wizards/src/engine/mod.rs`
- `core/crates/wizards/src/engine/eval.rs`
- `core/crates/wizards/src/engine/eval_expr.rs`
- `core/crates/wizards/tests/dsl_eval.rs`
- `core/crates/wizards/tests/expr_eval.rs`
- `core/crates/wizards/tests/shelf_gen.rs`
- `core/crates/wizards/tests/box_gen.rs`
- `core/crates/wizards/tests/leather_gen.rs`
- `tests/golden/wizards/shelf_input_01.json`
- `tests/golden/wizards/shelf_expected_01.json`
- `tests/golden/wizards/box_input_01.json`
- `tests/golden/wizards/box_expected_01.json`
- `tests/golden/wizards/leather_input_01.json`
- `tests/golden/wizards/leather_expected_01.json`
- `docs/status/trace-index.json`
- `docs/verification/S13-WIZARDS-IMPL-005.md`

## Commands & Evidence
### Preflight / History
- `git status --porcelain`
- `git fetch --all --prune`
- `git checkout -b feature/s13-wizards-impl-005`
- `git log -n 30 --oneline`
- `git branch -vv`
- `git rev-parse HEAD` → `a9a09fde15227c5f01768ff527112574c8c81d82`
- `cargo test -q` (repo root) → fail (`Cargo.toml` not found at repo root; workspace is under `core/`)

### Tests
- `cargo test -p craftcad_wizards` (in `core/`) → pass
- `cargo test -p craftcad_library` (in `core/`) → pass
- `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library` (in `core/`) → pass
- `cargo test` (in `core/`) → fail at existing `craftcad_io_bridge/tests/compat_matrix_golden.rs::compat_report_golden` (known unrelated failure)

## Spec alignment
- `wizard_policy.md`: seed（derived、golden比較ではseed正規化）。
- `templates/*.template.json`: `generation_steps` の `op` に従い `PartsDraft` 生成。
- `required_presets`: テンプレ先頭を決定的に採用（v1）。
- `box_wizard` の `w_expr/h_expr` は安全式評価（数値/参照/四則/括弧のみ）で解禁。

## Notes / Risks
- leatherのfeaturesは大量：goldenはfeatures空に正規化し、統計で検証（境界/件数）。
- boxはv1簡易（front panelのみ）。将来6面/継手はStep6/7/追加タスクで拡張可。

## Allowlist self-check
- Allowed paths内のみ。
- 削除なし。

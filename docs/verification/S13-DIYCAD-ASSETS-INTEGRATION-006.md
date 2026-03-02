# Verification: S13-DIYCAD-ASSETS-INTEGRATION-006

## Goal
`.diycad` documentへ資産参照（used_presets/templates/wizard_runs）を互換維持で統合し、migration+salvage+テストで壊れないことを保証する。

## SSOT location (evidence)
- `.diycad` schema SSOT path: `docs/specs/schema/diycad/document.schema.json`
- migration implementation path: `core/serialize/src/lib.rs` (`normalize_document_json`)

## Changed files
- `docs/specs/schema/diycad/document.schema.json`
- `core/serialize/schemas/document.schema.json`
- `core/serialize/src/lib.rs`
- `core/serialize/tests/diycad_document_migration.rs`
- `core/serialize/tests/schema_lint.rs`
- `tests/compat/diycad_document_migration.rs`
- `tests/golden/diycad_document_v1.json`
- `core/crates/diycad_format/src/lib.rs`
- `core/crates/diycad_format/tests/salvage_basic.rs`
- `core/crates/presets/src/reasons.rs`
- `core/crates/presets/src/salvage.rs`
- `core/crates/presets/src/lib.rs`
- `core/crates/presets/tests/salvage.rs`
- `core/crates/presets/Cargo.toml`
- `core/crates/wizards/src/run_record.rs`
- `core/crates/wizards/src/lib.rs`
- Document初期化追従のための既存テスト/ベンチ更新（`core/**`）
- `docs/status/trace-index.json`
- `docs/verification/S13-DIYCAD-ASSETS-INTEGRATION-006.md`

## Commands & Evidence
### Preflight / History
- `git status --porcelain`
- `git fetch --all --prune`
- `git checkout -b feature/s13-diycad-assets-integration-006`
- `git log -n 40 --oneline`
- `git branch -vv`
- `git rev-parse HEAD` → `c3f91eb283143affa818aff0be562d0f75c9f8ae`
- `cargo test -q` (repo root) → fail (`Cargo.toml` not found at repo root; workspace is under `core/`)

### Tests
- `cargo test -p craftcad_presets` → pass
- `cargo test -p craftcad_serialize` → pass
- `cargo test -p craftcad_diycad_format --test salvage_basic` → pass
- `cargo test -p craftcad_wizards` → pass
- `cargo test` (core workspace) → fail at existing `craftcad_io_bridge/tests/compat_matrix_golden.rs::compat_report_golden`

## Spec alignment
- `used_presets` / `used_templates` / `wizard_runs` を schema に追加。
- 旧 `schema_version:1` document は normalize/migrate で `schema_version:2` + 新配列フィールド補完。
- salvage は欠落新フィールドを自動補完して read-only 復旧を継続。
- presets salvage API で missing 時の builtin fallback version 候補を返却可能。

## Notes / Risks
- UI導線（wizard実行をproject保存へ接続）は後続スプリントで接続。
- `tests/compat/diycad_document_migration.rs` は互換証跡ファイルとして追加（実行系は `core/serialize/tests/diycad_document_migration.rs`）。

## Allowlist self-check
- Allowed paths内のみ。
- 削除なし。

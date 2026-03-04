# Verification: S20-STEP3
## Goal
ReasonCatalog正本に基づくError UX（原因+提案+ジャンプ）が決定的に生成され、Actionsが実行可能な“完成形の骨格”になっていること。

## History evidence
- HEAD: 215447e01e6b30c9b3eaf91fd66ffd6461ca6874
- Branch: feature/s20-step3-001
- Evidence:
  - docs/specs/ux/error_ux_policy.md の確認: `test -f docs/specs/ux/error_ux_policy.md && echo exists:...`
  - docs/specs/errors/catalog.json の確認: `test -f docs/specs/errors/catalog.json && echo exists:...`
  - `git fetch origin` は remote 未設定で失敗
  - `git log --oneline -n 30` / `git branch -vv` で S20 Step1/2 と前提履歴を確認
  - 既存SupportZip/consent/redaction の呼び出し口（調査メモ）:
    - `core/crates/diagnostics/src/support_zip.rs` に consent attach/load と redaction 適用が存在
    - `attach_consent` / `build_in_dir` で `consent_store.load()` と `redactor.redact_json(...)` を使用
    - `consent.json` を zip に出力（line 206 付近）
    - `git log -- core/crates/diagnostics/src/support_zip.rs` では S17/S18 で品質・security統合が入っている

## Changed files
- apps/desktop/src/app/error_ux/mod.rs
- apps/desktop/src/app/error_ux/reason_catalog.rs
- apps/desktop/src/app/error_ux/error_mapper.rs
- apps/desktop/src/app/error_ux/error_panel.rs
- apps/desktop/src/app/error_ux/actions.rs
- apps/desktop/src/app/error_ux/history.rs
- apps/desktop/src/app/error_ux/tests.rs
- apps/desktop/src/app/mod.rs（最小配線）
- docs/status/trace-index.json
- docs/verification/S20-STEP3.md

## What/Why
- UI直書きを排し、ReasonCatalog+i18nキーを正本として “原因/提案/ジャンプ” を必ず提示する。
- 最大3アクション・Actions最上部・詳細折りたたみ・コピー導線で “詰まらない” を担保する。

## Tests
- `rustfmt apps/desktop/src/app/mod.rs apps/desktop/src/app/error_ux/*.rs` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_desktop error_ux::tests` (fail: package not found in this repo layout)

## Determinism/Safety
- 同一inputで display_hash 一致（mapping_is_deterministic）を unit test で追加。
- unknown action kind は mapper で安全に無視（unknown_action_kind_is_ignored）。
- contextはredacted済み前提。保存/コピーモデルは reason_code/severity/job/op/hash + redacted context のみ。

## Allowlist self-check
- allowed pathsのみ
- 削除なし

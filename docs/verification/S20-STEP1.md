# Verification: S20-STEP1
## Goal
UX SSOT（onboarding/error_ux/mode/navigation/sample）を docs/specs/ux に確定し、SSOT lintで破壊を停止できること。

## History evidence
- HEAD: 06d82977faabd2acf5f65c93d32258a21fc8a30a
- Branch: feature/s20-step1-001
- Notes:
  - preflight: `git status --short`(clean), `git status -sb`, `git rev-parse --abbrev-ref HEAD`, `git rev-parse HEAD`
  - preflight: `git fetch origin` は remote 未設定のため失敗（`fatal: 'origin' does not appear to be a git repository`）
  - preflight: `git checkout feature/s20-step1-001 || git checkout -b feature/s20-step1-001`
  - history: `git log --oneline -n 20` / `git branch -vv` で S19/S18/S17 系コミットを確認
  - baseline CI-equivalent: `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint -- --nocapture` が 14 tests pass
  - `spec_ssot_lint.rs` の既存責務（blame/log要約）:
    - S12 で IO SSOT lint が導入
    - S13 で presets/templates/library lint を追加
    - S16 で perf budgets 検証を追加
    - S17 で diagnostics SSOT lint を追加
    - S18 で security SSOT lint を追加
    - S19 で a11y/i18n lint を追加

## Changed files
- docs/specs/ux/README.md
- docs/specs/ux/onboarding_flow.md
- docs/specs/ux/error_ux_policy.md
- docs/specs/ux/mode_policy.md
- docs/specs/ux/navigation_policy.md
- docs/specs/ux/sample_library.md
- core/serialize/src/spec/ux_ssot.rs
- core/serialize/src/spec/mod.rs
- core/serialize/src/lib.rs
- core/serialize/Cargo.toml
- core/serialize/tests/spec_ssot_lint.rs
- docs/status/trace-index.json
- docs/verification/S20-STEP1.md

## What/Why
- UXを“仕様＝契約”として固定し、後続Step（Onboarding/ErrorUX/Modes/Jobs統合）でUIが場当たりにならないようにする。
- SSOT YAML block を lint で検証し、必須キー欠落・遷移重複・未知Action をPRで停止する。

## Spec alignment (SSOT)
- B02-01 モード責務と遷移: mode_policy.md の状態機械/遷移表を正本化し、(from,event)一意を検証。
- B02-02 操作一貫性: keys契約を明記し、今後のinput_routerがこの契約に従う前提を固定。
- B02-03 初回体験: onboarding_flow.md で成功体験と完了条件（OpLog/Job完了）を固定。
- B02-04 エラーUX: error_ux_policy.md で ReasonCatalog正本・Action最大3・実行可能Action契約を固定。

## Tests
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` (pass)
- negative check（契約破壊検知）:
  - `docs/specs/ux/mode_policy.md` に `(from,event)=(Select,ToolSelectExport)` 重複を一時挿入
  - `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` が想定通り fail
  - ファイルを復元し、再実行で pass

## Allowlist self-check
- 変更は allowed paths（docs/**, core/**）のみ。
- 削除なし（新規追加と追記のみ）。

## Determinism & Safety
- lint が “重複遷移” “未知Action” “SSOT欠落” を検知し、契約破壊を止めることを確認。

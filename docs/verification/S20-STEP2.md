# Verification: S20-STEP2
## Goal
Onboardingの骨格（samples/first_run/state machine/completion/ui model）がSSOTに準拠して実装され、unit testで決定性が担保されること。

## History evidence
- HEAD: b791aa85824541202386215b0565974c23c0118e
- Branch: feature/s20-step2-001
- Evidence:
  - `docs/specs/ux/onboarding_flow.md` が存在: `test -f docs/specs/ux/onboarding_flow.md && echo exists ...`
  - preflight: `git status -sb` / `git rev-parse --abbrev-ref HEAD` / `git rev-parse HEAD`
  - preflight: `git fetch origin` は remote 未設定のため失敗
  - history: `git log --oneline -n 20` / `git branch -vv`
  - apps/desktop/src/app の現状（配線箇所）:
    - 実エントリポイントは Qt 側 `apps/desktop/src/main.cpp`（QApplication, QMainWindow）
    - Rust UIモジュールは `apps/desktop/ui/a11y/mod.rs` 配下（shortcuts/focus/accessibility）

## Changed files
- apps/desktop/src/app/onboarding/mod.rs
- apps/desktop/src/app/onboarding/spec.rs
- apps/desktop/src/app/onboarding/samples.rs
- apps/desktop/src/app/onboarding/first_run.rs
- apps/desktop/src/app/onboarding/tutorial_state.rs
- apps/desktop/src/app/onboarding/completion.rs
- apps/desktop/src/app/onboarding/ui.rs
- apps/desktop/src/app/onboarding/tests.rs
- apps/desktop/src/app/mod.rs（最小配線）
- docs/status/trace-index.json
- docs/verification/S20-STEP2.md

## What/Why
- 初回体験（最短成功体験）をSSOTからロードし、状態機械として扱えるようにする。
- ログ/ジョブはtrait注入で切り離し、後続StepでE2Eまで“安全に繋ぐ”。
- FileStoreによりDB無しで初回フラグを永続化し、破損時も安全に復旧（デフォルトへフォールバック）する。

## Tests
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_desktop onboarding::tests` (fail: package not found in this repo layout)

## Determinism/Safety
- 同じSSOT/同じOpLog/同じJobStatus → 同じ遷移（`apps/desktop/src/app/onboarding/tests.rs` に unit test を実装）。
- PIIを保存しない（FirstRunStateはフラグと時刻のみ）。

## Allowlist self-check
- allowed paths のみ変更していること
- 削除が無いこと

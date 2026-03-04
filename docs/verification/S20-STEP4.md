# Verification: S20-STEP4
## Goal
Mode/NavigationがSSOT準拠の状態機械として実装され、入力一貫性と禁止遷移（job/dirty）を統一処理できること。

## History evidence
- HEAD: de2a3f93c13982d980023e75e3965b313133c09c
- Branch: feature/s20-step4-001
- Evidence:
  - docs/specs/ux/mode_policy.md / navigation_policy.md の存在確認:
    - `test -f docs/specs/ux/mode_policy.md && echo exists:docs/specs/ux/mode_policy.md`
    - `test -f docs/specs/ux/navigation_policy.md && echo exists:docs/specs/ux/navigation_policy.md`
  - `git fetch origin` は remote 未設定で失敗
  - `git log --oneline -n 30` / `git branch -vv` で履歴確認
  - Undo/Redo 実装箇所の調査メモ（blame要約）:
    - Desktopショートカット: `apps/desktop/src/input/shortcut_map.h` (`isUndo`/`isRedo`)
    - Desktop入力処理: `apps/desktop/src/canvas_widget.cpp` で `store_->undo/redo`
    - Desktop-FFI橋: `apps/desktop/src/doc_store.cpp` で `craftcad_history_undo/redo`
    - coreコマンド基盤: `core/commands/tests/*` に history undo/redo roundtrip群

## Changed files
- apps/desktop/src/app/modes/**
- apps/desktop/src/app/navigation/**
- apps/desktop/src/app/mod.rs（最小配線）
- docs/status/trace-index.json
- docs/verification/S20-STEP4.md

## What/Why
- モード遷移を表駆動にし、場当たり分岐を排除して“同じ操作は同じ結果”を保証する。
- job_running/dirty/dialog/focus を状態に持ち、禁止遷移を一箇所で統一する。
- navigation（breadcrumbs/backstack/deep-link）をSSOTに従い提供し、Error UXのジャンプ導線を作る。

## Tests
- `rustfmt apps/desktop/src/app/mod.rs apps/desktop/src/app/modes/*.rs apps/desktop/src/app/navigation/*.rs` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_desktop modes::tests navigation::tests` (fail: package not found in this repo layout)

## Determinism/Safety
- 同一(state,event)→同一遷移（same_event_same_result）を unit test 追加。
- job_runningでDraw拒否（job_running_denies_draw）を unit test 追加。

## Allowlist self-check
- allowed pathsのみ
- 削除なし

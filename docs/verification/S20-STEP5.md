# Verification: S20-STEP5
## Goal
Nest/Export/Open/Save等の重い操作がジョブ導線として統合され、進捗/キャンセル/失敗復帰（ErrorPanel）/成功通知（Onboarding）が常時保証されること。

## History evidence
- HEAD: de2a3f93c13982d980023e75e3965b313133c09c
- Branch: feature/s20-step5-001
- Evidence:
  - Sprint16 JobQueue 実装箇所の調査メモ（path/型/blame）:
    - `core/crates/jobs/src/queue.rs` に `JobQueue`, `JobHandle`, `cancel`, `progress`, `try_result` が実装されている
    - `git log -- core/crates/jobs` は `c48958b S16: implement deterministic jobs queue core with contract tests` を示す
    - `git blame core/crates/jobs/src/queue.rs` 先頭で deterministic priority queue の設計コメントと `JobQueueConfig`/`JobHandle` を確認
  - Nest/Export/Open/Save のenqueue入口メモ:
    - 本Stepで `apps/desktop/src/app/mod.rs` に `enqueue_nest_job / enqueue_export_job / enqueue_open_job / enqueue_save_job` を追加
    - 入口は `jobs_ux::JobUxEvent::Enqueue` に集約し、UIが重い処理を直接実行しない導線に統一
  - preflight:
    - `git fetch origin` は remote 未設定で失敗
    - `git log --oneline -n 40`, `git branch -vv` 実行

## Changed files
- apps/desktop/src/app/jobs_ux/**
- apps/desktop/src/app/mod.rs（配線）
- apps/desktop/src/app/modes/transitions.rs（job_running同期入口）
- apps/desktop/src/app/error_ux/mod.rs（Retry接続補助）
- apps/desktop/src/app/onboarding/mod.rs（成功通知hook）
- docs/status/trace-index.json
- docs/verification/S20-STEP5.md

## What/Why
- UIが詰まらないため、重い操作は必ずジョブ化し進捗/キャンセルを提供する。
- 失敗時はReasonCode→ErrorUXで原因/提案/ジャンプを提示し自己解決へ導く。
- 成功時はOnboarding完了条件に接続し初回成功体験を短縮する。

## Tests
- `rustfmt apps/desktop/src/app/mod.rs apps/desktop/src/app/onboarding/mod.rs apps/desktop/src/app/error_ux/mod.rs apps/desktop/src/app/modes/transitions.rs apps/desktop/src/app/jobs_ux/*.rs` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_lint_ux_specs -- --nocapture` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize` (pass)
- `cargo test --manifest-path core/Cargo.toml -p craftcad_desktop jobs_ux::tests` (fail: package not found in this repo layout)

## Determinism/Safety
- `jobs_ux::tick` は active job snapshot を単一順序で評価し、同一状態遷移で同一effect列を返す。
- `last_enqueued_req` + `last_failed_job` により RetryLastJob を決定的に再enqueue可能。
- contextはredacted前提、SupportZipはconsent必須（Step3/18方針と整合）。

## Allowlist self-check
- allowed pathsのみ
- 削除なし

# SPRINT14-STEP7 Verification

## Goal
Sprint14 Step7/8 の品質ゲートとして、`.diycad` の open/save 互換性・決定性・耐破損性・E2E バッチ挙動を PR で常時検証できる状態にした。  
本対応は **バイナリ fixture 非依存** を前提に、すべての `.diycad` 入力をテスト実行時に tempdir で生成する。  
Golden/Compat/Fuzz/Determinism/E2E の5系統を `scripts/ci/run_all.sh` に統合し、短時間で再現可能な失敗シグネチャ比較を導入した。  
Golden 更新は `GOLDEN_ACCEPT=1` のときだけ許可し、通常実行では差分検出で失敗する。  
壊れた入力（manifest欠落・broken part・future schema・bad zip 混在）でもクラッシュせず、read-only/summary 出力などの期待挙動を確認する。  

## Changed files (`git diff --name-only`)
- core/crates/diycad_format/src/open.rs
- core/crates/io_support/src/lib.rs
- core/crates/migration/src/step.rs
- core/crates/wizards/Cargo.toml
- scripts/ci/run_all.sh
- tests/compat/compat_open.rs
- tests/determinism/open_signature.rs
- tests/e2e/migrate_verify_batch.rs
- tests/fuzz/diycad_open_fuzz.rs
- tests/golden/diycad_open_save/mod.rs
- tests/golden/diycad_open_save/golden_signatures.json
- tests/golden/io_roundtrip/expected/compat/compat_report.json

## History evidence
- preflight clean check: `git status --porcelain` は空、`CLEAN:0` を確認。  
- branch: `feature/sprint14-step7-001` を作成して checkout。  
- HEAD at start: `94ae6f9ff3533a1ae9f9e72f1fdbededf57f5a78`。  
- recent log/reflog/branch は実行済み（`git log -n 20`, `git log --graph ... -n 60`, `git branch -vv`, `git reflog -n 30`）。

## Commands executed
- `cd core && GOLDEN_ACCEPT=1 cargo test -q -p craftcad_wizards --test diycad_open_save`
- `cd core && cargo test -q`（初回、既存ゲート差分確認のため実行）
- `scripts/ci/run_all.sh`（修正後に再実行し、最終で PASS）
- `cd core && cargo test -q -p craftcad_wizards --test diycad_open_save --test compat_open --test diycad_open_fuzz --test open_signature --test migrate_verify_batch`

## Self-check
- Allowed-path check: OK（変更は `docs/**`, `core/**`, `tests/**`, `scripts/**` のみ）。
- バイナリ非依存: OK（本タスク内で `.diycad` バイナリをコミットしていない。すべてテスト内 tempdir 生成）。

## PR作成手順（運用）
- PR本文は 5 行以内の短文テンプレで作成する。  
- クライアント側で PR 作成が不安定な場合は Safari(Web) から作成する。  
- 手動バイナリが必要な場合の配置仕様はタスク記載の「10) 手動追加仕様」を参照する。

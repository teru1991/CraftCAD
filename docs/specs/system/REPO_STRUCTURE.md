# REPO_STRUCTURE

最終更新: 2026-02-28  
ステータス: SSOT（Single Source of Truth）草案

## 1. ディレクトリ責務

- `core/`
  - ドメインモデル、幾何計算、`.diycad` の読み書き、決定性担保ロジック
- `apps/desktop/`
  - Qt デスクトップ UI、入出力操作、表示・編集導線
- `apps/mobile/`
  - Flutter モバイル UI、入出力操作、表示・編集導線
- `core/crates/diycad_ffi/`
  - Desktop 連携向け C ABI 境界（Qt などから呼び出す薄いブリッジ）
- `tools/`
  - 開発補助スクリプト、CI 補助、フォーマット・検証支援
- `docs/specs/system/`
  - システム SSOT（本書含む）
- `docs/specs/security/`
  - セキュリティ仕様（脅威モデル、入力検証方針）
- `docs/specs/observability/`
  - ログ、メトリクス、障害解析方針
- `docs/specs/release/`
  - リリース手順、バージョニング、互換性ポリシー
- `testdata/`
  - 再現性のある固定入力/期待値データ

## 2. 依存方向（必須）

- 許可:
  - `apps/desktop -> core`
  - `apps/desktop -> core/crates/diycad_ffi`（C ABI 経由）
  - `apps/mobile -> core`
  - `tools -> (core, apps)`
- 禁止:
  - `core -> apps/*`
  - `apps/desktop -> apps/mobile`
  - `apps/mobile -> apps/desktop`
  - UI 層同士の相互依存

## 3. 設計原則

- Core を唯一のビジネスロジック境界とする
- UI 層は表示/入力変換に集中し、計算の本体を持たない
- ファイルフォーマット（`.diycad`）の解釈実装は Core へ集約

## 4. 禁止事項

- DB 導入（SQLite を含む）
- 「一時的だから」という理由での非決定的処理の恒久化
- UI 層での独自フォーマット解釈（Core と重複する実装）
- SSOT 未更新のまま仕様実装を先行すること

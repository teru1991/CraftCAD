# TECH_STACK_AND_BOOTSTRAP

最終更新: 2026-02-28  
ステータス: SSOT（Single Source of Truth）草案

## 1. 目的

CraftCAD の採用技術、導入順、非採用技術を固定し、初期実装の判断ブレをなくす。

## 2. 採用技術（確定）

- Core: **Rust**
- Desktop: **Qt**（C++/CMake ベース）
- Mobile: **Flutter**（Dart）
- 正本フォーマット: **`.diycad`**（zip + manifest + data + assets）

## 3. 非採用（明示）

- **DB を採用しない**（RDB / NoSQL / SQLite を含む）
  - 理由:
    - データ正本は `.diycad` 単体で自己完結させるため
    - 実行環境差異と移行コストを抑えるため
- 共有状態を DB 前提で設計しない
- 「ローカルキャッシュ目的の SQLite」も導入しない

## 4. ブートストラップ導入順

1. リポジトリ骨格（ディレクトリ、共通規約ファイル）
2. Core 最小クレート作成（型定義、入出力境界、決定性ポリシー）
3. `.diycad` v0 の読み書き PoC（Core）
4. Desktop シェル（Qt）: ファイルを開く/保存する最小経路
5. Mobile シェル（Flutter）: 同等の最小経路
6. 決定性テストと互換性テストの定着
7. CI で lint/test/package の最小パイプライン化

## 5. 初期運用ルール

- 実装は Core を起点に行い、UI 層は Core の機能を呼び出すのみとする
- 仕様変更は先に `docs/specs/system/` の SSOT を更新してから実装する
- 互換性に影響する変更は `DECISION_LOG.md` に必ず記録する

# PROJECT_FORMAT

最終更新: 2026-02-28  
ステータス: `.diycad` v0 仕様案（SSOT 草案）

## 1. 位置づけ

`.diycad` は CraftCAD プロジェクトの**正本**であり、外部 DB を必要としない自己完結アーカイブとする。

## 2. コンテナ形式

- 拡張子: `.diycad`
- 実体: ZIP アーカイブ
- 文字コード: UTF-8
- パス区切り: `/`

## 3. ルート構造（v0）

```text
project.diycad
├─ manifest.json
├─ data/
│  ├─ document.json
│  └─ components/
└─ assets/
   ├─ images/
   └─ fonts/
```

## 4. `manifest.json`（最小要件）

必須フィールド:

- `schema_version` : 文字列（例: `"0"`）
- `format` : 文字列固定（`"diycad"`）
- `created_by` : 生成元情報（アプリ名/バージョン）
- `created_at` : 生成日時（ISO-8601）
- `entrypoint` : 既定データへの相対パス（例: `"data/document.json"`）

## 5. v0 シリアライズ規約

- JSON は UTF-8
- キー順序は安定化（辞書順推奨）
- 浮動小数の出力桁・丸め規則は `DETERMINISM.md` に準拠
- ZIP 内ファイルの列挙順は安定化する

## 6. 互換性と移行方針

- 後方互換を原則とする
- `schema_version` により読み込み分岐を行う
- 破壊的変更時は移行器（migrator）を提供する
- 移行は `vN -> vN+1` の段階方式を基本とする
- 仕様変更は `DECISION_LOG.md` に記録する

## 7. 非目標（v0）

- DB 連携を前提にした差分保存
- サーバ常駐状態に依存する整合性機構

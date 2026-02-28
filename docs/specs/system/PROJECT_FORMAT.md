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

## 3. ルート構造（v0 最小）

```text
project.diycad
├─ manifest.json              # 必須
├─ data.json                  # 必須（現時点は空構造を許容）
└─ assets/
   └─ thumbnail.png           # 任意
```

## 4. `manifest.json`（v0 最小要件）

必須フィールド:

- `schema_version`: 文字列（v0 は `"0"`）
- `app_version`: 生成元アプリのバージョン
- `units`: 単位（例: `"mm"`）
- `created_at`: 生成日時（ISO-8601）
- `modified_at`: 更新日時（ISO-8601）

## 5. `data.json`（v0 最小要件）

- 図面データ本体
- v0 では空構造（例: 空配列/空オブジェクト）を許容

## 6. 実装 API（v0）

- `save(path, project)`
  - `.diycad`（ZIP）として `manifest.json` と `data.json` を保存
  - `thumbnail.png` は存在時のみ保存
- `load(path)`
  - ZIP を読み込み、`manifest.json`/`data.json` を復元
  - `schema_version != "0"` は不一致エラー

エラー最小セット:

- 壊れた ZIP（invalid zip）
- `manifest.json` 欠落
- `schema_version` 不一致
- `data.json` 欠落

## 7. v0 シリアライズ規約

- JSON は UTF-8
- キー順序は安定化（辞書順推奨）
- 浮動小数の出力桁・丸め規則は `DETERMINISM.md` に準拠
- ZIP 内ファイルの列挙順は安定化する

## 8. 互換性と移行方針

- 後方互換を原則とする
- `schema_version` により読み込み分岐を行う
- 破壊的変更時は移行器（migrator）を提供する
- 移行は `vN -> vN+1` の段階方式を基本とする
- 仕様変更は `DECISION_LOG.md` に記録する

## 9. 非目標（v0）

- DB 連携を前提にした差分保存
- サーバ常駐状態に依存する整合性機構

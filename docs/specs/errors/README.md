# ReasonCode SSOT

`catalog.schema.json` を唯一の構造定義、`catalog.json` を唯一の値定義として扱います。

## 運用ルール
- すべての失敗/未対応/劣化は ReasonCode を返す。
- FATAL はクラッシュを意味しない。処理を安全停止し、救出手順を提示する。
- `doc_link` はリポジトリ相対パスで必須。
- UI 文言は code をキーに i18n 層で解決する。

## i18n
- `core/i18n/locales/*.json` に `reason.<code_lowercase>` キーを追加して表示文言を管理します。

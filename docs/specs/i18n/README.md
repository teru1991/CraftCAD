# i18n SSOT (Sprint19)
このディレクトリは UI 文字列の正本（SSOT）です。アプリ側に文字列を直書きしません。

## 正本
- 翻訳リソース: apps/desktop/i18n/resources/{ja,en}.json
- 仕様: policy.md / keys.md / units.md
- CI契約: required_keys.json（主要導線キーは必ず両言語に存在）

## 追加手順（新しいUI文字列）
1. keys.md の命名規約に従い key を決める
2. apps/desktop/i18n/resources/ja.json と en.json に同じ key を追加
3. 主要導線に含まれるなら required_keys.json にも追加
4. core の SSOT lint テストを通す（欠落/重複/不正は CI で落ちる）

## よくあるCIエラー
- MISSING_KEY: required_keys.json にあるキーが ja/en に存在しない
- UNKNOWN_KEY_FORMAT: "UI." で始まらない翻訳キーが含まれている
- SCHEMA_INVALID: schema 違反（値がstringでない、長すぎる 等）
- DUPLICATE_KEY: 同一key重複（JSON上は起きにくいが生成ミス検知）

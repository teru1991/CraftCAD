# Drawing Style SSOT

このディレクトリは図面スタイルの Single Source of Truth (SSOT) です。

## 対象
- 線種（line styles）
- 線幅（line weights）
- 色規約（color policy）
- フォントファミリとフォールバック
- 文字高さ・矢印スタイル
- 丸め規約（mm/inch）
- 寸法配置規約

## 運用ルール
1. **互換**: `style_ssot.json` は schema 準拠を必須とし、互換を壊す変更は `version` を更新する。
2. **追加手順**:
   - `style_ssot.schema.json` に定義を追加
   - `style_ssot.json` に値を追加
   - `core/serialize/tests/spec_ssot_lint.rs` で lint と重複検出を更新
3. **禁止事項**:
   - 既存キーの削除・意味変更を `version` 更新なしで行わない
   - line style 名に大文字やハイフンを使わない（`^[a-z][a-z0-9_]*$`）
   - ランタイムで未定義のデフォルト値を埋めることで SSOT を迂回しない

## 決定性
`RenderPlan` を使う処理は、入力順・ソート順・丸め規約を固定し、同一入力で同一出力となること。

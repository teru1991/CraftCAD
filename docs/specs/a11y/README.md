# Accessibility SSOT (Sprint19)
このディレクトリはアクセシビリティ要件の正本（SSOT）です。

## 目的
- 主要フローは「マウス無し（キーボードのみ）」で完走できる
- フォーカス移動が迷子にならない（順序/可視化/トラップ）
- ショートカットは衝突しない（重複禁止）
- 高DPI/フォント拡大/色弱配慮で視認性が成立する

## 正本
- policy.md: 全体規約（キーボード、フォーカス、読み上げ）
- shortcuts.md: 人間向け一覧
- required_shortcuts.json: CIの正本（最低限、必須ショートカット）
- hidpi.md / color.md: 視認性規約

## 追加/変更手順
1) shortcuts.md を更新（モード依存/衝突禁止を明記）
2) 必須なら required_shortcuts.json も更新（欠落はCIで落ちる）
3) apps 側の shortcuts テーブルを更新
4) tests（shortcut_coverage）で衝突/欠落がないことを確認

# UX SSOT (Sprint20 / B02-01..04)
このディレクトリは「UXは仕様＝契約」を固定するSSOT。
変更は必ずPRで行い、core/serialize/tests/spec_ssot_lint.rs が機械検証する。

## ルール
- 各specは “SSOT YAML block” を必須とする（<!-- SSOT:BEGIN --> ... <!-- SSOT:END -->）
- UI文言の直書きは禁止。必ず i18n key か ReasonCatalog 経由にする（本Sprintでの契約）
- モード遷移は状態機械1箇所が真実。場当たり分岐は禁止。

## 対象ファイル
- onboarding_flow.md : 初回体験（サンプル→木取り→出力）を固定
- error_ux_policy.md : エラー表示（原因/提案/ジャンプ/ログ）を固定
- mode_policy.md : モード状態機械と遷移表を固定
- navigation_policy.md : breadcrumbs/backstack/deep-link を固定
- sample_library.md : 同梱サンプル要件（互換/サイズ/更新）を固定

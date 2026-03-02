# Support Matrix（人間可読）— SSOT補助

このファイルは docs/specs/io/support_matrix.json の説明用（機械の正は json）。
実装・判定は必ず support_matrix.json を参照すること。

## 概要
- format: dxf / svg / json
- direction: import / export
- feature: entity_*, attr_*, unit_*, external_reference 等
- level: supported / best_effort / not_supported

## Best-effort の約束
- 必ず reason_codes を列挙（警告に出す）
- 必ず action を明記（approx/fallback/convert_or_approx 等）
- 実装は reason_codes をそのまま warnings に積む（contextを付ける）

## Not supported の約束
- action: drop / replace を明記
- 必ず reason_codes を列挙（落とさないための説明）

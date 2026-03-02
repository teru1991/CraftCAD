# Presets Compatibility Policy

- N-2読み込み保証（例: 現行`schema_version=3`なら `1..=3` を読める）
- 破壊的変更は (1) 新ID化 or (2) 明示migration必須
- built-in資産は常に互換維持（削除しない、deprecated扱いは可）
- 依存解決不能時はReasonCode（`PRESET_DEP_MISSING_*`）で説明する（このタスクはSSOTのみ、ReasonCode実装はStep2以降）

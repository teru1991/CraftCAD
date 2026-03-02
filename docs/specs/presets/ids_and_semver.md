# Preset / Template IDs and SemVer

## Preset ID命名規約
- pattern: `^[a-z0-9][a-z0-9_\-]*$`
- 予約語禁止: `"con"`, `"prn"`, `"aux"`, `"nul"`（Windows由来。将来のファイル名衝突回避）
- 最大長: 64

## Template ID命名規約
- pattern: `^[a-z0-9][a-z0-9_\-]*$`
- 最大長: 64

## SemVer運用
- `version`: `MAJOR.MINOR.PATCH`（必須）
- 互換: 破壊は `MAJOR`
- 旧版読込保証: N-2（`schema_version`単位）

## schema_version運用
- 1から開始
- `schema_version`の増分は「読み込み互換が破られる可能性」がある変更のみ
- forward compatibilityのため unknown fields は保持可能（Rust側は `serde(flatten)` 余地）

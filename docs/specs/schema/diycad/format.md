# .diycad package format

- 拡張子 `.diycad` は ZIP コンテナ
- 文字エンコード UTF-8
- パス区切り `/`

## Layout
- `/manifest.json`
- `/document.json`
- `/parts/*.json`
- `/nest_jobs/*.json`
- `/assets/*`

## Determinism
- JSON 出力は key sort + pretty/固定丸め
- Nest 系は seed/epsilon/順序を明示保存
- checksum は任意だが記録推奨

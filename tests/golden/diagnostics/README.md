# Diagnostics Golden (Sprint17)

このディレクトリは diagnostics 生成物の“契約”を固定します。
更新は必ず `--accept` を付けた golden_update を使って行い、意図しない変更はCIで落ちます。

## Files
- joblog_sample.json
- oplog_sample.json
- reason_summary_sample.json
- repro_sample.md
- support_zip_manifest.json (zip内ファイル一覧 + sha256/sizeの期待)

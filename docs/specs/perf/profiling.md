# Profiling運用ルール

1. 変更前後で `PerfReport` を取得し、dataset_id/seed/schema_version を固定して比較する。
2. 最適化は budget 超過の根因が可視化された span のみ対象にする。
3. CIでは当面 WARN 運用、運用が安定次第 ERROR へ切替する。
4. 診断ZIPへ `perf_report.json` を必ず添付し、サポート時の再現情報として使う。

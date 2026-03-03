Support ZIP Contract (Sprint17)

診断ZIPは“安全で役に立つ提出物”です。構成と同意条件を固定します。

## ZIP root layout (fixed)
必須（常に入れる）
- joblog.json
- reason_summary.json
- ssot_fingerprint.json
- perf_report.json            # 無い場合は省略可（存在するなら必須）
- oplog.json                  # OpLogが存在する場合は必須（無いなら省略可）

任意（ユーザー同意がtrueのときのみ）
- project_snapshot.diycad      # include_project_snapshot=true のときのみ
- inputs/                      # include_inputs_copy=true のときのみ（既定false）
  - <input_id>.<ext?>          # input_id由来（PII無し）。拡張子は推測しなくても良い

## Size limits (must enforce via limits API)
- max_total_bytes: (from limits profile)
- max_single_file_bytes: (from limits profile)

## If exceeding limits
- 超過する場合はトランケート/除外し、joblog.timeline に warning ReasonCode を追加する
  - DIAG_ZIP_TRUNCATED

## Consent (must reflect in consent_snapshot)
- include_project_snapshot: default false
- include_inputs_copy: default false
- telemetry_opt_in: default false

## Naming (must not include PII)
- support-<job_id>-<timestamp_utc>.zip
- job_idはUUID等（PII無し）

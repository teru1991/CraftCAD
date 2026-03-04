# Error UX Policy SSOT (B02-04)
原則: エラーは ReasonCatalog が正本。UI文言直書き禁止。

## SSOT
<!-- SSOT:BEGIN -->
kind: error_ux_policy
version: 1
ui_contract:
  # 表示順（ユーザーが詰まらない）
  order:
    - Actions
    - Title
    - Detail
    - Links
    - Debug
  actions_max: 3
  detail_collapsed_by_default: true
  pii_safe: true
required_actions:
  # UI側が実行できるアクションの“契約”。実装が無い場合 lint で FAIL。
  - OpenDocs
  - OpenSettings
  - CreateSupportZip
  - RunMigrateTool
  - RetryLastJob
  - JumpToEntity
  - DuplicateSampleAsProject
mapping_contract:
  # ReasonCatalogの各entryは title_key / detail_key / actions を持てる
  require_title_key: true
  require_detail_key: true
logging:
  # 表示したエラーを必ず OpLog/JobLog に残す
  record_on_show: true
  record_fields:
    - reason_code
    - severity
    - job_id
    - op_id
    - debug_ref
safety:
  redaction_required: true
  consent_required_for_supportzip: true
<!-- SSOT:END -->

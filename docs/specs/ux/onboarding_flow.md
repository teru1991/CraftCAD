# Onboarding Flow SSOT (B02-03)
目的: 初見ユーザーが「サンプル→木取り→出力」まで自力到達できる“最短成功体験”を固定する。

## SSOT
<!-- SSOT:BEGIN -->
kind: onboarding_flow
version: 1
flow_id: onboarding_sample_to_print
entrypoints:
  - first_run_auto
  - help_menu_rerun
  - settings_rerun
completion:
  # すべて満たしたら完了
  all_of:
    - op: OpenSample
      args:
        sample_id: sample_shelf_project
    - job: NestJob
      status: Succeeded
    - job: ExportJob
      status: Succeeded
steps:
  - id: open_sample
    title_key: ux.onboarding.open_sample.title
    body_key: ux.onboarding.open_sample.body
    required:
      any_of:
        - op: OpenSample
          args:
            sample_id: sample_shelf_project
    next: view_parts
    can_skip: true
    links:
      docs: ux/onboarding#open_sample
  - id: view_parts
    title_key: ux.onboarding.view_parts.title
    body_key: ux.onboarding.view_parts.body
    required:
      any_of:
        - op: OpenPanel
          args: { panel: Parts }
    next: run_nest
    can_skip: true
    links:
      docs: ux/onboarding#parts
  - id: run_nest
    title_key: ux.onboarding.run_nest.title
    body_key: ux.onboarding.run_nest.body
    required:
      any_of:
        - job: NestJob
          status: Succeeded
    next: open_export
    can_skip: false
    links:
      docs: ux/onboarding#nest
  - id: open_export
    title_key: ux.onboarding.open_export.title
    body_key: ux.onboarding.open_export.body
    required:
      any_of:
        - op: OpenPanel
          args: { panel: Export }
    next: do_export
    can_skip: true
    links:
      docs: ux/onboarding#export
  - id: do_export
    title_key: ux.onboarding.do_export.title
    body_key: ux.onboarding.do_export.body
    required:
      any_of:
        - job: ExportJob
          status: Succeeded
    next: done
    can_skip: false
    links:
      docs: ux/onboarding#finish
policy:
  allow_rerun: true
  allow_skip: true
  # “コピーして編集” 導線を必須とする（サンプル誤編集防止）
  sample_open_mode: read_only
  recommended_actions:
    - DuplicateSampleAsProject
<!-- SSOT:END -->

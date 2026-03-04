# Navigation Policy SSOT (B02-02)
breadcrumbs/backstack/deep-link を統一し、迷子復帰を保証する。

## SSOT
<!-- SSOT:BEGIN -->
kind: navigation_policy
version: 1
breadcrumbs:
  # 表示階層の契約（例）
  pattern:
    - Project
    - Parts
    - Part
    - NestJob
    - Export
backstack:
  enabled: true
  max_depth: 50
  esc_does_not_pop: true
deep_links:
  # Error UX の JumpToEntity が到達できる対象
  targets:
    - Entity
    - Part
    - NestJob
    - ExportResult
<!-- SSOT:END -->

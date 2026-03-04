# Mode Policy SSOT (B02-01, B02-02)
目的: モード遷移は “同じ操作は同じ結果” を守る。状態機械で一本化し例外増殖を防ぐ。

## SSOT
<!-- SSOT:BEGIN -->
kind: mode_policy
version: 1
modes:
  - Select
  - Draw
  - Edit
  - Dimension
  - Annotate
  - Nest
  - Export
  - Settings
  - Support
keys:
  Esc: CancelToSelect
  Enter: Commit
  Tab: NextFocus
  Undo: CmdOrCtrlZ
  Redo: ShiftCmdOrCtrlZ
global_guards:
  # 例: “ジョブ実行中”は一部遷移を禁止し、復帰導線（キャンセル/待機）を出す
  - id: deny_when_job_running
    condition: { job_running: true }
    denies:
      - to: Draw
      - to: Edit
      - to: Dimension
      - to: Annotate
    required_recovery_actions:
      - CancelActiveJob
      - ShowJobProgress
transitions:
  # (from, event) は一意。重複したら lint で FAIL。
  - from: "*"
    event: KeyEsc
    to: Select
    side_effects: [CancelPreview]
  - from: Draw
    event: KeyEnter
    to: Select
    side_effects: [CommitTool]
  - from: "*"
    event: ShortcutUndo
    to: "*"
    side_effects: [Undo]
  - from: "*"
    event: ShortcutRedo
    to: "*"
    side_effects: [Redo]
  - from: Select
    event: ToolSelectDraw
    to: Draw
    side_effects: [BeginTool]
  - from: Select
    event: ToolSelectNest
    to: Nest
    side_effects: [OpenPanelNest]
  - from: Select
    event: ToolSelectExport
    to: Export
    side_effects: [OpenPanelExport]
dialogs:
  # “未保存破棄”は確認が必須（遷移そのものにガードを掛ける）
  confirm_on_discard_dirty: true
<!-- SSOT:END -->

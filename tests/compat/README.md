# Compat tests (N-2) — binary-free
このディレクトリは **N-2互換の証拠** を固定するための資産です（資産/出力はテキストのみ）。

## 対象
- projects/: 旧版 project_exchange JSON（N-2）を **open→migrate→validate** できること
- presets/: 旧版 preset JSON を **migrate→resolve** できること
- templates/: 旧版 template JSON を **validate→execute(dry-run可)** できること
- io/: 外部由来のSVG（+ 任意でASCII DXF）を **import→normalize** できること

## 重要ルール
- **資産の削除は禁止**。不要になった場合は `deprecated` と記して残す。
- 互換劣化がある場合は **ReasonCodeで説明**（サイレント破壊禁止）。
- **Binary-free**: *.json/*.svg/*.txt 以外をコミットしない。

## forward-incompatible
`projects/forward_incompatible_project.json` は “未来版” を想定した資産です。
- 期待挙動:
  - 可能なら read-only として開く（閲覧可能）
  - それが無理なら **CP_FORWARD_INCOMPATIBLE** と明確なReasonCodeで拒否する
- どちらであっても **クラッシュ禁止**。

## 失敗時の出力
失敗したテストは `failure_artifacts/compat/<case_id>/` に以下（テキストのみ）を出します:
- meta.json（compat_case_id/種別/path/schema_version/migrate_path等）
- repro_input.json/svg/txt（入力の完全コピー）
- actual.json（生成物がある場合）
- diff.txt（比較がある場合）
- reason_codes.json（ある場合）

# .diycad format (SSOT)

Purpose
- Project canonical data container (DBなしの正本)。アプリは設定/キャッシュのみ。
- 互換: N-2 読み込み保証。書き出しは最新schema_versionのみ。段階migration必須。
- 破損/欠落/未知フィールド: クラッシュ禁止。可能な限り部分復旧し read-only で開く。
- 決定性: 同一zip内容 + 同一schema_version なら同一OpenResult (read_only/warnings含む)。

Container
- File extension: .diycad
- Container: ZIP (store/deflate どちらでも良い。読み込みは両対応)
- Encoding: UTF-8 (ファイル名/JSON)
- Line endings: LF in generated files (推奨)

Required entries (MUST)
- /manifest.json
- /document.json

Optional entries (MAY)
- /parts/*.json
- /nest_jobs/*.json
- /assets/**
  - /assets/thumbnails/**    (任意。消しても再生成できるものは入れないのが推奨)
  - /assets/previews/**      (任意。出力プレビューなど)
  - /assets/diagnostics/**   (任意。PII禁止。ジョブログ要約等)
- /_migrations/**            (任意。dry-run report等。ただし巨大化に注意)

Path rules (SECURITY + determinism)
- No absolute paths. No drive letters. No backslash as separator.
- No traversal: ".." segments are forbidden.
- Max path length / depth is limited by limits policy (see integrity.md / limits in code).
- Enumeration order MUST be stable: sort by normalized path ascending (bytewise/UTF-8).

JSON files
- UTF-8 JSON text.
- Unknown fields:
  - strict_schema=true のときはエラー
  - strict_schema=false のときは「unknown fields」として警告し、読み込みは継続（可能なら）

Entrypoints
- manifest.json.entrypoints.document MUST reference the document JSON path (default: "document.json")
- document.json MAY reference parts/nest_jobs by IDs; loader must resolve best-effort.

Determinism contract
- When reading, scanning order is fixed by sorted path.
- Warnings list order: by (ReasonCode, path, kind) ascending.
- FailedEntry order: by (path) ascending.

Recovery / Autosave
- Autosave generations are NOT stored inside .diycad. They are app-local snapshots.
- Recovery UI uses recovery.md contract; the core recovery module provides list/restore primitives.

Integrity
- Optional manifest.content_manifest provides sha256+size per entry for corruption detection.
- If content_manifest is present and verification fails: open read_only=true and expose salvage actions.
- If content_manifest is absent: warn only (compat-first).

# Sprint14 Runbook: Project survival (.diycad)

## Scope
- 保存/復旧/移行/救出の運用手順と、失敗時の再現・診断導線をまとめる。

## Canonical data rules
- 正本は `.diycad`（DBなし）。アプリ側は設定/キャッシュのみ。
- キャッシュ（thumbnails/previews/index等）は消してよい（再生成可能）。

## When “project won’t open”
1. `open_package` の結果が `read_only=true` の場合
   - UI: 「読み取り専用で開かれました」を表示し、`salvage_actions` を提示する。
   - 推奨アクション:
     - `ExportSalvagedDocument` / `ExportSalvagedParts`
     - `GenerateDiagnosticsZip`（PII禁止）
     - `ResaveAsNewProject`（正規化して新規保存）
     - `SuggestMigrateTool`（旧版/互換外の場合）
2. manifest欠損
   - `document.json` を探索で開ける可能性がある。
   - まず読み取り専用で開き、直ちに `ResaveAsNewProject` を行う。
3. integrity mismatch（sha/size）
   - 改竄/破損の可能性。`read_only` で開き、救出書き出し→新規保存を推奨。

## Autosave / Recovery
- autosave は app-local の世代（generations）。
- クラッシュ後は「最後の良い世代」を復旧候補として必ず提示する。
- `*.tmp` が残っていても復旧は可能（無視/掃除される）。

## Migration tool (`diycad-migrate`)
- 安全な移行:
  - `diycad-migrate INPUT.diycad --to latest --output OUT.diycad`
  - `--dry-run`: 差分要約のみ
  - `--verify`: 移行後の検証（strict）
  - `--batch`: 一括処理 + JSON summary
- in-place は原則禁止（必要なら例外手続きで導入）。

## Golden tests (updating fixtures)
- Golden mismatch が出たら:
  - 仕様変更が意図通りか SSOT を確認
  - 更新が正当なら `GOLDEN_ACCEPT=1` で更新し、変更理由をPRに明記
- 無闇に更新しない（互換破壊の検知器）。

## Fuzz repro
- Fuzz が落ちた場合:
  - `tests/fuzz/repro/` の入力（もし保存されていれば）を使って再現
  - `open_package` の `ReasonCode` を確認し、limits/parse/salvage のどれが原因か切り分け

## Security
- zip bomb/深いパス/巨大エントリは limits で拒否される。
- 外部参照（URL/ファイルパス）を不用意に追わない（本スプリントは追わない）。

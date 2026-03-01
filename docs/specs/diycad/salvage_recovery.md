# .diycad Salvage & Partial Recovery 仕様

## 概要

`.diycad` ファイルは ZIP アーカイブ形式であり、manifest、document.json、parts/\*.json、nest_jobs/\*.json などから構成されています。

本仕様では、以下のシナリオで **部分復旧（salvage）** を実施します：

- ZIP ファイル自体は開けるが、個別エントリが破損している場合
- JSON スキーマバリデーションに失敗した場合
- Document は正常だが、一部の Part や NestJob が読み込めない場合

## スコープ

### 復旧対象
- **Document JSON**: スキーマ違反時の自動正規化
- **Part JSON**: 個別ロード時の best-effort 復旧
- **NestJob JSON**: 個別ロード時の best-effort 復旧

### 復旧対象外（既存エラーで終了）
- **ZIP ファイル** が破損している（read エラー）
- **Manifest JSON** が無効である

## 処理フロー

### 段階 1: Manifest バリデーション
```
try:
  manifest = read_manifest()
  validate_manifest(manifest)
catch:
  return Error(SERIALIZE_PACKAGE_CORRUPTED)
```

### 段階 2: Document JSON パース & 正規化
```
try:
  document_json = read_file("document.json")
  document_value = parse_json(document_json)
  (document, report) = salvage_document(document_value)
  if report.recovered == false:
    return Error(...)
catch:
  return Error(SALVAGE_DOCUMENT_MALFORMED)
```

### 段階 3: Parts 個別ロード（best-effort）
```
parts = []
parts_failed = []
for part_file in manifest.parts:
  try:
    part = load_part(part_file)
    parts.push(part)
  catch:
    parts_failed.push({
      file: part_file,
      reason: error_reason,
      reason_code: SALVAGE_PART_RECOVERABLE
    })

if parts.is_empty() and manifest.parts.len() > 0:
  return Error(SALVAGE_TOO_MUCH_DAMAGE)
```

### 段階 4: NestJob 個別ロード（best-effort）
```
nest_jobs = []
nest_jobs_failed = []
for job_file in manifest.nest_jobs:
  try:
    job = load_nest_job(job_file)
    nest_jobs.push(job)
  catch:
    nest_jobs_failed.push({...})
```

## ReasonCode 一覧

| Code | 説明 | Severity | Retryable |
|------|------|----------|-----------|
| `SALVAGE_DOCUMENT_MALFORMED` | Document JSON がスキーマ違反 | ERROR | true |
| `SALVAGE_PART_RECOVERABLE` | 個別 Part の復旧（失敗）  | WARN | true |
| `SALVAGE_PARTIAL_LOSS` | Part/NestJob の一部が失われた | WARN | true |
| `SALVAGE_TOO_MUCH_DAMAGE` | 復旧不可（全 Part が無効） | ERROR | false |

## Document 正規化ルール

Document がスキーマ違反の場合、以下のルールで自動補填：

```rust
if document.materials is missing:
  document.materials = []

if document.settings is missing:
  document.settings = {}

if document.outline is missing:
  return Error(SALVAGE_DOCUMENT_MALFORMED)  // 必須フィールド

if document.units is missing:
  document.units = "mm"  // デフォルト値
```

## 返り値構造

```rust
pub struct SalvageResult {
    pub document: Document,
    pub parts_loaded: usize,
    pub parts_failed: Vec<FailedPart>,
    pub nest_jobs_loaded: usize,
    pub nest_jobs_failed: Vec<FailedNestJob>,
    pub warnings: Vec<(String, ReasonCode)>,
    pub read_only: bool,  // 部分復旧した場合は true
}

pub struct FailedPart {
    pub filename: String,
    pub reason: String,
}

pub struct FailedNestJob {
    pub filename: String,
    pub reason: String,
}
```

## ユーザーへの通知

Salvage が実施された場合：

1. **ダイアログ表示**
   - 「このプロジェクトは部分的に破損していたため、復旧できる部分のみ読み込みました」
   - 読み込まれた Part 数 / 総数
   - 読み込まれた NestJob 数 / 総数

2. **UI 状態**
   - 読み取り専用モード有効
   - 警告アイコン表示
   - 操作不可な領域をハイライト

3. **ログ出力**
   - 失敗した各ファイルの理由
   - 出力された ReasonCode

## テスト例

### ケース 1: Document missing required fields
入力: `corrupt_missing_fields.diycad`
```json
// document.json
{
  "outline": [...],
  // materials と settings が欠落
}
```
期待: 自動補填で復旧、warnings に SALVAGE_DOCUMENT_MALFORMED は含まない

### ケース 2: Part 1つ破損、2つ正常
入力: `corrupt_partial_parts.diycad`
- parts/part_1.json: OK
- parts/part_2.json: 無効な JSON
- parts/part_3.json: OK

期待:
- parts_loaded = 2
- parts_failed = 1 (part_2.json)
- warnings に SALVAGE_PARTIAL_LOSS を含む
- read_only = true

### ケース 3: Manifest 自体が壊れている
入力: `corrupt_manifest.diycad`

期待: SERIALIZE_PACKAGE_CORRUPTED で終了（salvage なし）

### ケース 4: 全 Parts が無効
入力: `corrupt_all_parts.diycad`

期待: SALVAGE_TOO_MUCH_DAMAGE で終了


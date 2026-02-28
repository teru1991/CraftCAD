# CraftCAD Project Format: .diycad v1（SSOT）

## 目的
- DBなし運用のため、プロジェクトの正本は `.diycad` ファイル。
- 壊れても説明可能（ReasonCode）で復旧行動に繋げる。

## 形式
`.diycad` は ZIP パッケージ。

必須エントリ:
- `manifest.json`
- `data/document.json`
- `assets/`（ディレクトリ。中身は任意）

## manifest.json
- Schema: `core/serialize/schemas/manifest.schema.json`
- 役割:
  - schema_version（マイグレーションの根拠）
  - document_path / assets_path（将来の拡張）

## data/document.json
- Schema: `core/serialize/schemas/document.schema.json`
- 役割:
  - Document / Layer / Entity / Part / NestJob の SSOT

## バージョニング
- `manifest.schema_version` と `document.schema_version` は同一世代で運用する（v1）
- 将来の互換:
  - 読めない場合は `SERIALIZE_UNSUPPORTED_SCHEMA_VERSION` を返す
  - 読めるが不足がある場合は schema validation を通らず `SERIALIZE_SCHEMA_VALIDATION_FAILED`

## 破損・不正の扱い
- ZIPが開けない / 必須エントリ不足 → `SERIALIZE_PACKAGE_CORRUPTED`
- JSONが壊れている / schema違反 → `SERIALIZE_SCHEMA_VALIDATION_FAILED`
- decode失敗 → `SERIALIZE_PACKAGE_CORRUPTED`（debugに原因文字列）

## Note
- UI表示: Reason.user_msg_key + params
- ログ: Reason.debug（errors配列、path、値、schema id など）

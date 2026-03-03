Privacy / Redaction Policy (Sprint17)

診断ログ・診断ZIPは「サポート提出」用途に耐える必要がありますが、個人情報(PII)は含めません。

## 1. 原則
- 最小収集: 再現に必要な情報のみを保存する
- 既定でプロジェクト本体(.diycad)は含めない
- パス・入力テキスト・ユーザー識別子は保存しない（必要ならredacted/hashed）
- 同意(Consent)がある場合のみ、任意項目を追加できる（プロジェクトスナップショット等）

## 2. 禁止（絶対に入れてはいけないもの）
- 絶対パス/相対パス（例: C:\Users\..., /Users/...）
- OSアカウント名、PC名、メールアドレス、電話番号、住所
- 任意テキスト入力（注釈本文・ラベル文字列等）の“生”保存
- 外部URL（例: file://, http(s)://）の“生”保存

## 3. 許可される表現（安全な代替）
- source_hint_redacted: 文字列は redaction 済みで、PIIを含まない短いヒント（例: "drag_drop", "open_dialog", "clipboard"）
- sha256: 入力ファイルや成果物の内容ハッシュ（パスは不要）
- params_redacted: 仕様で定義されたキーのみを持つJSON（値は短く、PIIを含まず、上限で切り詰め）

## 4. Redaction要件（実装が守るべき契約）
- すべての「文字列」フィールドは、PIIを含まないことを保証する（redactorを必須にする）
- 文字列は最大長を制限する（limits）
- JSONは“許可キーのホワイトリスト”で出力する（未知キーは禁止、必要ならschemaを更新）

## 5. 同意（Consent）
- consent_snapshot に同意状態を保存する（include_project_snapshot/include_inputs_copy/telemetry_opt_in）
- include_project_snapshot / include_inputs_copy は既定 false

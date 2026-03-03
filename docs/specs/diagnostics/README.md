Diagnostics SSOT (Sprint17)

このディレクトリは「診断・ログ・サポート提出物」の契約(SSOT)です。
診断は“製品機能”であり、再現可能性・安全性(PII排除)・ユーザーコントロール(保持/削除)を最優先とします。

## 目的（非交渉）
- “再現できる報告” を最短で作れる
- 個人情報を含めない（既定でプロジェクト本体を含めない）
- 失敗しても落ちない（診断生成失敗=ReasonCodeで説明して続行）
- ユーザーが保持/削除/保存先を管理できる

## SSOTファイル一覧
- joblog.schema.json: JobLog（正本）— 再現に必要な環境/入力/Reason/タイムライン
- oplog.schema.json: OpLog（最小再生）— Action/Command単位の履歴（UIイベント生ログは禁止）
- repro_template.md: Issueに貼れる再現テンプレ（生成物）
- support_zip.md: 提出用ZIPの構成（必須/任意・同意条件・サイズ上限）
- retention_policy.md: 保持/削除/容量上限の契約
- privacy.md: 個人情報を含めない規約（禁止事項・redaction要件）

## 運用ルール（重要）
- “この契約に反するログ/ZIP” はCIで落とします（spec_ssot_lint）。
- 互換: schema_version を joblog/oplogに明示し、破壊的変更は互換ポリシーに従い手順化する。
- 再現性: determinism_tag（seed/eps/round/ordering）を常に出力する。

## 禁止事項（抜粋）
- 生のファイルパス、ユーザー入力テキスト、ユーザー名/メール、ネットワーク先URL、OSユーザー名の保存は禁止。
- UIイベントの生ログ（raw mouse/touch/key events）の保存は禁止（ノイズ・PII・肥大化）。

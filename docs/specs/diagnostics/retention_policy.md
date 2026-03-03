Retention Policy (Sprint17)

診断ログ・診断ZIPはユーザーのストレージを消費するため、保持・削除・容量上限の契約を固定します。

## Defaults (SSOT)
- default_keep_days: 14
- max_total_bytes: 2147483648   # 2 GiB
- max_items: 50

## Cleanup timing
- on_app_start: true
- periodic: "weekly"  # 実装都合で後続Sprintで変更可だが、契約は維持
- manual: true

## Deletion order (deterministic)
- 古い順（created_at asc）で削除
- 同時刻の場合は id の辞書順

## Safety
- 削除失敗はクラッシュ禁止（WARNとして残す）
- ユーザーが delete_item(id), delete_all() を呼べる

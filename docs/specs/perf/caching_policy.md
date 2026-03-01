# Caching / Job化ポリシー

- UIスレッドで重い計算を行わない。必ずジョブキュー経由で実行する。
- 新しい入力が来たら古い再構築ジョブを cancel する。
- キャッシュは dataset_id + schema_version + seed をキーにする。
- 低メモリ時はフル解像キャッシュを保持せず、ReasonCodeでユーザーに説明する。

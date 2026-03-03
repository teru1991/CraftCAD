# Golden tests
このディレクトリは Golden（回帰）資産を保持する。

- `inputs/`: 入力資産（SSOT: `tests/datasets/manifest.json` が参照）
- `expected/`: 期待資産（更新はローカルのみ）

## CI のルール
- CI は比較のみを行う（生成禁止）。
- 期待値更新はローカルの `tools/golden_update --accept` のみ許可。

## ハーネス
- 共通ロジックは `core::testing::golden_harness` に集約する。
- 失敗時は `failure_artifacts/<dataset_id>/` に再現情報（dataset_id/seed/eps/round/limits_ref/input_sha）と差分を出す。

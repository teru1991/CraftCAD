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

## Minimum set (Sprint15) — binary-free edition
このリポジトリのGolden最小セット（PRゲートの土台）は以下（入力/期待値はテキストのみ）:
- io_roundtrip_smoke（SVG/JSON import）
- io_export_reimport_smoke（import→export→reimport）
- project_json_open_save_smoke（プロジェクト交換JSONでopen/saveを回帰）
- wizard_shelf_smoke（wizard生成）

### 更新手順（ローカルのみ）
1) manifestの dataset を追加/更新（seed/eps/round/limits_ref を確定）
2) 入力 fixtures を `tests/golden/inputs/**` に追加（*.json/*.svg/(任意)ASCII *.dxf のみ）
3) 期待値を生成して固定（CI禁止）:
   - `./scripts/testing/run_golden.sh --accept --tags smoke`
4) `tests/golden/expected/**` の差分理由をPR本文に書く（回帰か仕様変更か）

NOTE:
- 期待値更新は PR ごとに1回を原則とし、差分説明（why）を必ず添付する。
- Step3では `.diycad(zip)` などバイナリ資産をコミットしない。
- `.diycad` の回帰は “バイナリ許可ステップ” で別途行う。

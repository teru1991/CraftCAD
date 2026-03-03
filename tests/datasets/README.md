# Datasets Manifest (SSOT)
この `tests/datasets/manifest.json` は **テスト資産の正本（SSOT）** です。
- すべての Golden / Determinism / Compat / E2E / Fuzz（短時間ゲート含む）は、最終的にこの台帳を起点に構成されます。
- **datasetの削除は禁止**（互換/回帰の証拠が失われるため）。不要になった場合は `tags` に `deprecated` を付け、CIの実行対象から外す運用にします。

## 期待値更新（Golden更新）
期待値（`tests/golden/expected/**`）の更新は **ローカルでのみ** 行います。
- `tools/golden_update --accept`（※ツール名は既存のSSOTに従う）
- CIでは **絶対に生成しません**（比較のみ）。

## 失敗時に必ず出す再現情報
各テスト/ゲートは失敗時に最低限、以下が揃うこと：
- dataset_id
- seed
- determinism: epsilon / round_step / ordering_tag
- limits_ref
- 入力path と（存在する場合）sha256
- ReasonCode一覧（順序固定：将来のGoldenで固定）

## パス安全性（重要）
manifestに書けるpathは **相対パスのみ** です。
- 絶対パス禁止
- `..` 禁止（path traversal禁止）
- `tests/golden/inputs/` もしくは `tests/golden/expected/` 配下のみ許可（Step1でLint化）

## フィールドの意味（要約）
- `seed`: 再現性の鍵（u64）
- `determinism.epsilon`: 幾何比較の許容誤差（>0, <=0.01）
- `determinism.round_step`: 丸め単位（>0, <=0.01）
- `ordering_tag`: 順序固定の識別子（将来、順序揺れ検知に使用）
- `limits_ref`: limitsプロファイル名（解決は後続Stepで追加してよい）

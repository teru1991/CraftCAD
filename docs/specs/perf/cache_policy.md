# Cache Policy SSOT

## 1. CacheKey は決定性材料のみ
- dataset_id, seed, eps, round_step, inputs_sha, options_hash, ssot_version_hash
- HashMap の順序に依存する値は禁止（必要なら正規化してハッシュ化）

## 2. 無効化ポリシー
- SSOT のハッシュが変わったら無効化
- schema_version が変わったら無効化
- input hash が変わったら無効化

## 3. メモリ上限
- LRU + サイズ上限（bytes 見積もり必須）
- 上限超過で破棄（ノイズ防止のため WARN は出さない）

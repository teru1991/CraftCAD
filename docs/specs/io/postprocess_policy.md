# Postprocess Policy — SSOT（加工機向け最適化 / 決定性）

## ssot_meta（このブロックは lint が検証する。削除禁止）
ssot_meta:
  policy_id: postprocess_policy
  required_keys: [origin_policy, join_eps, dedupe_eps, tiny_segment_min_len, stable_tiebreak, path_order]
  version: 1

## 1. 目的
export 前に、外部ソフト・加工機が受けやすい形へ「意味を壊さない範囲で」整形する。
- 原点移動
- パス結合
- 重複除去
- パス順序最適化（決定的）

## 2. ポリシー定義（値はSSOT注入、ここは意味を固定）
- origin_policy:
  - Keep：座標を維持
  - MoveToZero：bbox.min を (0,0) に平行移動（Reason: IO_FALLBACK_024）
- join_eps：端点距離がこの値以下なら結合（Reason: IO_FALLBACK_024）
- dedupe_eps：同一候補判定（Reason: IO_FALLBACK_024）
- tiny_segment_min_len：最小長未満を除去（Reason: IO_FALLBACK_024）
- path_order：近傍貪欲 + 安定タイブレーク（乱数禁止）
- stable_tiebreak：距離 → 座標 → ID → 種類 の順で安定決定する

## 3. 禁止事項
- 乱数による順序最適化は禁止
- HashMapの列挙順依存は禁止（必ず安定ソート）
- join/dedupe の候補選択が “実行環境で揺れる” 実装は禁止

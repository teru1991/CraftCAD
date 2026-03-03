# Memory Policy SSOT

## 1. 目的
- 巨大入力でも OOM で落ちない（limits と整合）
- 可能なら RSS/peak を計測し、budgets で回帰検知する

## 2. 劣化モード
- 上限に近づいたら “品質を落として継続” を優先できる設計にする（PERF_DEGRADED_MODE）
- ただし結果の意味が変わる場合は明示（ReasonCode）

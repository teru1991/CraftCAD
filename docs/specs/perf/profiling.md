# Perf / Profiling SSOT

この文書は「測れない最適化を禁止」し、常に原因追跡できる状態を維持するための規約です。

## 1. 目的
- 変更が “速くなった/遅くなった” を再現可能な数字で比較できる
- ボトルネックが span 名で特定できる
- 計測オーバーヘッドが小さく、DoSにならない

## 2. span 命名規約（固定）
形式: `domain.action[.sub]`
例:
- io.import.parse
- io.import.map
- io.normalize
- io.postprocess.join
- diycad.open.preflight
- diycad.open.salvage.parts
- render.frame.build_primitives
- render.frame.draw
- jobs.queue.wait

### 禁止
- 動的な値（IDやファイル名）を span 名に含めない
- 同じ処理に違う名前を付けない（比較不能になる）

## 3. 収集データの上限（DoS対策）
- spans は最大 N 件（実装側で固定上限を持つ）
- 1 span の記録も最大 N サンプルまで（p95近似のため）

## 4. 比較の基本単位
- dataset_id（tests/datasets/manifest.json の SSOT）
- determinism_tag（seed/eps/round など）

## 5. 期待する運用
- まず PerfReport を生成し、budgets.json の合格ラインで回帰検知
- 改善 PR には「どの span がどれだけ改善したか」を必ず添える

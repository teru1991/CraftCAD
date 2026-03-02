# SVG sample_01 — 期待挙動（Supported / Best-effort / Not supported）

## 目的
SVGの代表入力に対し、互換レベルとwarningsを固定する。

## 例の構成（想定）
- path(M/L/C)：Supported
- path(A)：Best-effort（Arc変換 or 近似）
- text：Best-effort（フォントはhint）
- 外部参照（xlink/href）：Not supported（遮断）

## 期待する挙動
- 外部参照は IO_IMAGE_REFERENCE_DROPPED を warnings に出し、参照は取り込まない
- Aコマンドは IO_CURVE_APPROX_APPLIED を出す（support_matrix参照）
- import後は normalize を必ず通す

# Curve Approximation Policy — SSOT（決定性）

## ssot_meta（このブロックは lint が検証する。削除禁止）
ssot_meta:
  policy_id: curve_approx_policy
  required_keys: [eps_default, eps_min, eps_max, max_segments, min_segments, arc_flatten, spline_flatten, max_iter]
  version: 1

## 1. 目的
外部形式の曲線（Spline/Bezier/EllipticalArc等）を、必要に応じてポリライン近似（flatten）する際の規約を固定する。
- 決定性：同入力→同点列
- 安全性：過剰分割を防ぐ
- 説明可能性：適用時は必ず ReasonCode + context に eps/segments を記録

## 2. 既定値（determinism SSOT が最終正）
このファイルは “規約の意味” を固定する。実際の値は determinism SSOT を参照して注入する（ただし lint は “存在と形” を検査する）。

- eps_default: 0.05mm 相当（例）
- eps_min: 0.001mm 相当（例）
- eps_max: 0.5mm 相当（例）
- min_segments: 8
- max_segments: 4096
- max_iter: 32

## 3. 近似方式
### 3.1 arc_flatten（円弧/楕円弧）
- 角度範囲と eps に基づき分割数を決定
- 分割数は [min_segments, max_segments] にクランプ
- 端点は必ず含める
- 出力点列の順序は入力パラメータ順（昇順）で固定

### 3.2 spline_flatten（Spline/Bezier）
- 反復分割（De Casteljau / 再帰分割相当）で、偏差が eps 以下になるまで分割
- 反復回数は max_iter を超えない
- 点列は入力曲線のパラメータ方向に従い、必ず同順序で出力

## 4. warnings / ReasonCode
- Best-effort で近似を適用した場合：IO_CURVE_APPROX_APPLIED を warnings に追加
- context は必須：
  - eps_used
  - segments
  - method（arc_flatten/spline_flatten）
  - clamp（min/maxに当たったか）

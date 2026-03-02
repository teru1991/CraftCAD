# DXF sample_01 — 期待挙動（Supported / Best-effort / Not supported）

## 目的
DXFの代表入力に対し、何が Supported / Best-effort / Not supported かを説明し、warnings（ReasonCode + context）の期待値を固定する。

## 例の構成（想定）
- LINE / LWPOLYLINE / ARC / CIRCLE：Supported
- TEXT/MTEXT：Best-effort（フォント等はhint）
- SPLINE：Best-effort（近似）
- HATCH/IMAGE/DIMENSION：Not supported（drop）

## 期待する挙動
1) import は落ちない
2) normalize を必ず通す（順序/丸め/閉路判定/NaN除外）
3) Best-effortは warnings に ReasonCode が必ず載る
   - SPLINE: IO_CURVE_APPROX_APPLIED（context: eps_used, segments, method）
   - TEXT: IO_TEXT_FALLBACK_FONT（context: fallback_font, size_guess）
4) Not supported は drop + warnings
   - HATCH: IO_HATCH_SIMPLIFIED（context: entity="HATCH"）

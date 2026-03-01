# 寸法ルール（人間可読）— Drawing Style SSOTの解釈規約

本書は、style_ssot.json の dimension / rounding / fonts / line_weights を、実装がどう解釈するかの約束を固定する。

## 1. 単位と丸め
- 内部単位：常に mm（units.internal = "mm"）
- 表示単位：units.display に従う（mm / inch）
- 丸め：
  - 長さ：rounding.length_step（例：0.1mm）にスナップして表示
  - 角度：rounding.angle_deg_step（例：0.5°）にスナップして表示
- 注意：内部計算は高精度で保持し、表示/出力で丸める（決定性のため丸め規約はSSOT固定）

## 2. 寸法種類（完成形で対応する対象）
- Linear：直列（Serial）/基準（Baseline）
- Angular：2線分角、または円弧中心角
- Radius / Diameter：円/円弧（中心推定は決定性規約に従う）

## 3. 寸法線・補助線の規約（style_ssot.dimension）
- ext_line_gap_mm：対象形状から補助線開始点までの“隙間”
- ext_line_overhang_mm：寸法線を超える補助線の“延長”
- dim_line_offset_mm：最初の寸法線の標準オフセット
- dim_line_step_mm：段数を増やす時の追加オフセット
- arrow：矢印タイプとサイズ（open/filled, size_mm）

## 4. テキスト配置の規約
- text_gap_mm：寸法線とテキストの最小離隔
- text_box_padding_mm：衝突判定用のテキストボックス余白
- テキスト背景（白抜き）は将来print presetで制御可能とする（Task3以降でRenderIRに反映）

## 5. 決定性（揺れ禁止）
- 寸法の候補選択は「優先度→距離→座標→ID」の安定タイブレークで決める
- 同じ入力（参照ジオメトリ・ヒント・style/template/preset）が同じ出力（IR）になること

## 6. 互換運用
- 既存プリセット（*_vN）の値を破壊的に変更しない
- 変更は必ず新しい *_v(N+1) を追加し、UI/プロジェクトが参照するIDを切り替える

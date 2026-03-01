# 注釈ルール（人間可読）— 穴径/面取り/リーダーの表記と配置規約

## 1. Hole Callout（穴径）
- 直径記号：必ず「⌀」を使用（フォントfallbackで表示崩れを防ぐ）
- 表記例：
  - ⌀10
  - ⌀10 x2（数量がある場合）
  - ⌀10 深さ5（深さがある場合：将来拡張）
- 丸め：style_ssot.rounding.length_step に従う
- 単位：units.display に従う（inch時はインチ表記に変換）

## 2. Chamfer / Round（面取り・R面）
- C面：C0.5 のように表記
- R面：R2 のように表記
- 丸め：style_ssot.rounding.length_step に従う

## 3. Leader（リーダー）
- デフォルト角度：style_ssot.dimension.leader.default_angle_deg
- 形状：アンカー→折れ点→テキスト
- 目標：読みやすさ優先。ただし決定性が最優先（候補探索は安定タイブレーク）
- 編集：テキスト位置/折れ点のドラッグは “ヒント” として保存し、再レンダでも同じ結果になること

# Units Policy (Metric/Imperial)
## 内部単位
v1 は内部単位を mm とする（計算・保存・入出力の中心単位）。

## 表示単位
- UnitSystem::Metric  => mm（小数1桁など、丸めは StyleSSOT/Determinism に従う）
- UnitSystem::Imperial => inch（v1は小数表記。分数は不要）

## ラベル
- mm: "mm"
- inch: "in"
- 数値と単位の間は 1スペース（例: "12.3 mm", "1.25 in"）
- -0.0 は 0.0 に正規化する

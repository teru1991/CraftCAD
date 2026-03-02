# Wizard Determinism Policy

- seed決定順位: explicit seed > dataset seed > ssot default seed
- ssot default seed: 20260101（固定整数）
- 生成結果の順序固定（parts/holes/annotations）
  - stable sort key: `(role, part_id, x_mm, y_mm, w_mm, h_mm)` を数値は丸め(0.001mm)して比較
- 丸め規約
  - すべてmm、内部はf64でも出力は 0.001mm に丸め
- “同入力→同出力” をテストで担保（Step7で追加）

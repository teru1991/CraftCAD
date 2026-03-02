# JSON sample_01 — 期待挙動（Round-trip基準）

## 目的
JSONを “内部モデルの正規表現” とし、Round-trip基準にすることを説明する。

## 期待する挙動
- import：validate → 復元 → normalize
- export：normalize → schemaに従って決定的に出力
- N-2読み込み保証（詳細は互換ポリシーと migration SSOT に従う）

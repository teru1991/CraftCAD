# Golden Policy (SSOT)
この文書は “Golden比較” を製品契約として固定する。

## 1. 目的
- PRごとに回帰を検知し、失敗時に必ず再現できる情報（dataset_id/seed/設定/入力）を揃える。
- CIでは比較のみ（生成禁止）。期待値更新は `tools/golden_update --accept` のみ。

## 2. JSON比較
### 2.1 正規化
- Object key: 昇順ソート（辞書順）
- Array order policy:
  - 既定は `Strict`（入力順を維持）
  - 明示的に unordered 指定がある場合のみ将来拡張で並び替え
- Float:
  - round_step で量子化（nearest）
  - -0.0 は 0.0 に正規化
  - NaN/Inf は不正（Reason として失敗扱い）

### 2.2 比較
- 正規化後の `serde_json::Value` 構造比較
- 不一致は deterministic な pretty diff を出す

## 3. SVG比較
### 3.1 正規化（保守的）
- タグ内 attribute は lexicographic に並べ替え（決定性）
- 数値は round_step 由来の丸めを適用
- 空白/改行は正規化（意味のない差を抑止）
- 解析失敗時は whitespace 正規化のみへフォールバック（決定性は維持）

### 3.2 比較
- 正規化後テキスト比較 + 差分出力

## 4. bytes（DXF等）
- bytes_hash 比較は最後の手段。
- 推奨は export→reimport→normalized_model.json 比較。

## 5. ReasonCode（説明責任）
- warnings.json から `code` 一覧を抽出
- dedupe + stable sort し Golden比較対象に含める

## 6. 失敗時出力（必須）
- dataset_id, seed, epsilon, round_step, ordering_tag, limits_ref
- inputs: path + （あれば）sha256
- expected/actual の保存
- diff.txt
- warnings_codes.json（あれば）

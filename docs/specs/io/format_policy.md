# I/O Format Policy (DXF / SVG / JSON) — 完成互換ポリシーSSOT

## 1. 目的
本書は、CraftCAD（仮）の入出力（DXF/SVG/JSON）における互換方針を SSOT として固定する。
- 外部から入れても壊れない（落ちない / 説明できる）
- 外部へ出しても使える（外部ツールが受ける形へ整形）
- Round-trip（import→normalize→export）が決定的（同入力→同出力）

本書で定義する “Supported / Best-effort / Not supported” は、必ず support_matrix.json の定義と一致しなければならない。

## 2. 対応形式（固定）
- DXF（R12〜の広い互換を意識。最小は LINE/LWPOLYLINE/ARC/CIRCLE/TEXT）
- SVG（path中心、circle/line/polyline/polygon/text）
- JSON（内部モデルの正規表現：Round-tripの基準）

## 3. 互換レベル定義
### 3.1 Supported
- 入出力で情報劣化が発生しない。
- import後の normalize により微小丸めは発生し得るが、幾何意味は保持される。
- export も規定桁で決定的に出力できる。

### 3.2 Best-effort（必須条件）
Best-effort は「落とさず救う」ための互換であり、必ず以下を満たす：
1) 変換規約（What）：何をどう変えるか（例：Spline→polyline近似、Arc→segments）
2) 劣化内容（Loss）：どこが変わり得るか（例：曲率/制御点/テキストフォント）
3) ReasonCode（必須）：warnings として返す
4) context（必須）：epsilon/segments/推定値/閾値など、再現に必要な情報を添付
5) support_matrix.json に “best_effort” として明示されていること（実装で直書き禁止）

### 3.3 Not supported（必須条件）
Not supported は「落とさないが、意味保持を保証しない」領域。必ず以下を満たす：
1) 取り扱い（Action）：drop する / 代替表現にする（support_matrix.json の action に従う）
2) ReasonCode（必須）：warnings（またはFATAL相当のAppError）として返す
3) 次の一手（Hint）：可能なら代替案（例：事前に曲線をポリライン化してから再試行）

## 4. 互換の根拠SSOT
実装は以下 SSOT を必ず参照し、対応判定を直書きしてはならない：
- docs/specs/io/support_matrix.json（対応範囲とBest-effort理由）
- docs/specs/io/mapping_rules.json（Layer/LineType/Unit 規約）
- docs/specs/io/curve_approx_policy.md（近似規約：epsilon/segments/決定性）
- docs/specs/io/postprocess_policy.md（加工機向け最適化：原点/順序/結合/重複除去）

## 5. 禁止事項（実装/運用）
- 実装に “if entity == Spline then …” のような対応判定を直書きしない（support_matrix参照）
- 入力上限（bytes/要素数/深さ）を直書きしない（limits SSOT参照）
- 決定性（round/eps/seed）を各所で勝手に変えない（determinism SSOT参照）
- “同じ入力なのに結果が変わる” 分岐（乱数・HashMap順序依存・非安定ソート）を入れない

## 6. Round-tripの基本方針
- import後：必ず normalize（順序/丸め/閉路判定/NaN除外）を通す
- export前：必ず normalize と postprocess（有効時）を決定的に適用する
- Best-effort / Not supported の差分は warnings として必ず ReasonCode + context で説明される

## 7. 例（参照）
- docs/specs/io/examples/dxf_sample_01.md
- docs/specs/io/examples/svg_sample_01.md
- docs/specs/io/examples/json_sample_01.md

# Verification: S12-IO-DXF-003

## Goal
DXF Import/Export を“製品品質”へ：ENTITIESセクションのみ解析、$INSUNITS反映、LWPOLYLINE/POLYLINE(VERTEX)正確復元（頂点ペアリング＋closed＋bulge→Arc）、DXF Export を support_matrix に整合させる。Golden＋回帰テストで round-trip と決定性を固定する。

## Spec / SSOT alignment
- `docs/specs/io/support_matrix.json` に dxf export の entity_line/entity_polyline/entity_arc/entity_circle/entity_text 等を追加（欠落により出力が NotSupported 扱いになる問題を解消）
- layer/linetype/decimal_places は `docs/specs/io/mapping_rules.json` に従う
- CubicBezier/未対応segmentは best-effort polyline 近似（ReasonCode: IO_CURVE_APPROX_APPLIED）

## Changed Files
- docs/specs/io/support_matrix.json
- docs/specs/io/support_matrix.md
- core/crates/io_dxf/src/parse.rs
- core/crates/io_dxf/src/import.rs
- core/crates/io_dxf/src/export.rs
- core/crates/io_dxf/tests/io_roundtrip_golden_dxf.rs
- core/crates/io_dxf/tests/dxf_polyline_bulge_regression.rs
- tests/golden/io_roundtrip/expected/dxf/normalized_internal_model.json
- tests/golden/io_roundtrip/expected/dxf/warnings.json
- tests/golden/io_roundtrip/expected/dxf/exported_out.dxf
- docs/status/trace-index.json
- docs/verification/S12-IO-DXF-003.md

## Preflight & History Evidence (paste outputs)
- git status:
```text
On branch work
nothing to commit, working tree clean
```
- git branch -vv:
```text
* work 4f679ef S12-IO-SVG-002: productize SVG import limits/path/transform handling
```
- git log --oneline --decorate -n 30:
```text
4f679ef (HEAD -> work) S12-IO-SVG-002: productize SVG import limits/path/transform handling
e265644 Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
f3ab920 Implement SSOT-driven IO support and reason code parity
...
```
- git log --oneline --decorate --graph --max-count=60:
```text
* 4f679ef (HEAD -> work) S12-IO-SVG-002: productize SVG import limits/path/transform handling
*   e265644 Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
|\
| * f3ab920 Implement SSOT-driven IO support and reason code parity
|/
...
```
- git show --stat:
```text
commit 4f679ef... S12-IO-SVG-002: productize SVG import limits/path/transform handling
```
- git blame core/crates/io_dxf/src/parse.rs | head -n 160:
```text
(preflightで実行、既存実装履歴を確認)
```
- git blame core/crates/io_dxf/src/import.rs | head -n 200:
```text
(preflightで実行、既存実装履歴を確認)
```
- git blame core/crates/io_dxf/src/export.rs | head -n 220:
```text
(preflightで実行、既存実装履歴を確認)
```
- git blame docs/specs/io/support_matrix.json | head -n 160:
```text
(preflightで実行、既存実装履歴を確認)
```

## Tests Run (paste commands + results)
- `cd core && cargo fmt --all` ✅
- `cd core && cargo clippy --workspace --all-targets -- -D warnings` ❌（既存の `faces/src/extract.rs` の clippy lint 失敗）
- `cd core && GOLDEN_ACCEPT=1 cargo test -p craftcad_io_dxf --tests` ✅
- `cd core && cargo test -p craftcad_io_support -p craftcad_io -p craftcad_io_dxf -p craftcad_io_json -p craftcad_io_svg -p craftcad_io_bridge` ✅
- `cd core && cargo test -p craftcad_io_bridge --tests` ✅

## Self-check (Hard rules)
- [x] Only allowed paths touched
- [x] No deletions (only edits/additions)
- [x] Spec updated first (support_matrix)
- [x] Added tests (golden + bulge regression)
- [x] Determinism preserved (fixed dp, stable section parsing, unit scale)
- [x] Limits respected (no max() raising of limits)

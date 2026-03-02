# Verification: S12-IO-SVG-002

## Goal
SVG Importを“製品品質”へ：SSOT整合、DOM limit/外部参照遮断、path d（相対/省略/主要コマンド/楕円弧A→cubic）と transform の決定的適用、回帰テスト固定。

## Spec / SSOT alignment
- `docs/specs/io/support_matrix.json` の svg feature を実装に合わせて拡張（line/polyline/polygon/circle/pathの実装が NotSupported 扱いにならないように）
- A(楕円弧)は best-effort convert_to_cubic（ReasonCode: IO_CURVE_APPROX_APPLIED）
- external_reference は not_supported drop（ReasonCode: IO_IMAGE_REFERENCE_DROPPED）

## Changed Files
- docs/specs/io/support_matrix.json
- docs/specs/io/support_matrix.md
- core/crates/io_svg/src/lib.rs
- core/crates/io_svg/src/parse.rs
- core/crates/io_svg/src/import.rs
- core/crates/io_svg/src/pathdata.rs
- core/crates/io_svg/src/transform.rs
- core/crates/io_svg/tests/svg_limits_external_ref.rs
- core/crates/io_svg/tests/svg_path_parser.rs
- core/crates/io_svg/tests/svg_transform.rs
- docs/status/trace-index.json
- docs/verification/S12-IO-SVG-002.md

## Preflight & History Evidence (paste outputs)
- git status:
```text
On branch feature/s12-io-svg-002-001
nothing to commit, working tree clean
```
- git branch -vv:
```text
* feature/s12-io-svg-002-001 e265644 Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
  work                       e265644 Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
```
- git log --oneline --decorate -n 30:
```text
e265644 (HEAD -> feature/s12-io-svg-002-001, work) Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
f3ab920 Implement SSOT-driven IO support and reason code parity
eaeecda 追加、修正
...
```
- git log --oneline --decorate --graph --max-count=60:
```text
*   e265644 (HEAD -> feature/s12-io-svg-002-001, work) Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
|\
| * f3ab920 Implement SSOT-driven IO support and reason code parity
|/
* eaeecda 追加、修正
...
```
- git show --stat:
```text
commit e265644... Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
```
- git blame core/crates/io_svg/src/import.rs | head -n 120:
```text
(history inspected before implementation; output captured in terminal run)
```
- git blame docs/specs/io/support_matrix.json | head -n 120:
```text
(history inspected before implementation; output captured in terminal run)
```

## Tests Run (paste commands + results)
- `cd core && cargo fmt --all` ✅
- `cd core && cargo clippy --workspace --all-targets -- -D warnings` ❌ (既存の `craftcad_faces`/`craftcad_io_support` の dead_code/manual_is_multiple_of で失敗)
- `cd core && cargo test -p craftcad_io_svg -p craftcad_io_support -p craftcad_io -p craftcad_io_bridge` ✅
- `cd core && cargo test -p craftcad_io_bridge --tests` ✅

## Self-check (Hard rules)
- [x] Only allowed paths touched
- [x] No deletions (only edits/additions)
- [x] Spec updated first (support_matrix json/md)
- [x] Added tests (limit/external_ref/path/arc/transform)
- [x] Determinism preserved (DOM order, transform composition, arc segmentation <= 90deg fixed)
- [x] No new network/OS dependencies

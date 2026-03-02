# Verification: S12-IO-SSOT-001

## Goal
Sprint12(I/O拡張)の土台として、SSOT（support_matrix.json / mapping_rules.json）を実装が正しく消費し、ReasonCodeを損なわずに扱える状態にする。

## Scope
- Fix: io_support SSOT parsing (mapping_rules shape), duplicate method removal, deterministic normalization
- Fix: craftcad_io::ReasonCode parity with support_matrix reason_codes (lossless)
- Update: SSOT lint tests to match current mapping_rules.json shape
- Add: regression tests for mapping_rules behavior and support_matrix reason mapping

## Changed Files
- core/crates/io_support/Cargo.toml
- core/crates/io_support/src/lib.rs
- core/crates/io_support/tests/io_contract_lint.rs
- core/crates/io_support/tests/support_matrix_reason_mapping.rs
- core/crates/io_support/tests/mapping_rules_behavior.rs
- core/crates/io/src/reasons.rs
- docs/specs/io/mapping_rules.md
- docs/status/trace-index.json
- docs/verification/S12-IO-SSOT-001.md

## SSOT Alignment Notes
- mapping_rules.json の実フィールド（max_len/forbidden_chars_regex/normalize/export 等）に Deserialize を追従させた
- support_matrix.json の reason_codes を ReasonCode enum でロスなく表現できるようにした

## Preflight & History Evidence (paste outputs)
- git status:
```
## feature/s12-io-ssot-001-001
 M core/crates/io/src/reasons.rs
 M core/crates/io_support/Cargo.toml
 M core/crates/io_support/src/lib.rs
 M core/crates/io_support/tests/io_contract_lint.rs
 M docs/specs/io/mapping_rules.md
 M docs/status/trace-index.json
?? core/crates/io_support/tests/mapping_rules_behavior.rs
?? core/crates/io_support/tests/support_matrix_reason_mapping.rs
```

- git branch -vv:
```
* feature/s12-io-ssot-001-001 eaeecda 追加、修正
  work                        eaeecda 追加、修正
```

- git log --oneline --decorate -n 20:
```
eaeecda (HEAD -> feature/s12-io-ssot-001-001, work) 追加、修正
845a676 Merge pull request #55 from teru1991/codex/add-curve-approximation-and-postprocessing-k6k8gd
d9be341 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-k6k8gd
6c04c8d Harden support matrix handling and improve SVG/DXF best-effort conversions
51a806b Merge pull request #54 from teru1991/codex/add-curve-approximation-and-postprocessing-jlyxob
51b76c6 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-jlyxob
69edce0 Add io_bridge diycad roundtrip integration and E2E save-reopen flow
4ab6c8a Merge pull request #53 from teru1991/codex/add-curve-approximation-and-postprocessing-9u5ddk
f28c5b8 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-9u5ddk
6a7f075 Implement io_svg/io_dxf parsers and deterministic IO support integration
57f23d9 Merge pull request #52 from teru1991/codex/add-curve-approximation-and-postprocessing-3rzvjl
b78ef97 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-3rzvjl
08f0b14 Implement io_json baseline with schema validation and golden roundtrip assets
12b1f97 Merge pull request #51 from teru1991/codex/add-curve-approximation-and-postprocessing
364e755 Add deterministic curve approximation and export postprocess pipeline
5f7aecd Merge pull request #50 from teru1991/codex/remove-ucel-mixed-in-files
f5da151 chore: remove accidentally included UCEL project files
7ce9d5f Merge pull request #49 from teru1991/codex/implement-automated-ci-green-loop
749a2b5 Harden Windows CI by installing Qt and robust Python launcher
e146322 Build desktop FFI automatically in build_desktop script
```

- git log --oneline --decorate --graph --max-count=40:
```
* eaeecda (HEAD -> feature/s12-io-ssot-001-001, work) 追加、修正
*   845a676 Merge pull request #55 from teru1991/codex/add-curve-approximation-and-postprocessing-k6k8gd
|\  
| *   d9be341 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-k6k8gd
| |\  
| |/  
|/|   
* |   51a806b Merge pull request #54 from teru1991/codex/add-curve-approximation-and-postprocessing-jlyxob
|\ \  
| * \   51b76c6 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-jlyxob
| |\ \  
| |/ /  
|/| |   
* | |   4ab6c8a Merge pull request #53 from teru1991/codex/add-curve-approximation-and-postprocessing-9u5ddk
|\ \ \  
| * \ \   f28c5b8 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-9u5ddk
| |\ \ \  
| |/ / /  
|/| | |   
* | | |   57f23d9 Merge pull request #52 from teru1991/codex/add-curve-approximation-and-postprocessing-3rzvjl
|\ \ \ \  
| * \ \ \   b78ef97 Merge branch 'main' into codex/add-curve-approximation-and-postprocessing-3rzvjl
| |\ \ \ \  
| |/ / / /  
|/| | | |   
* | | | |   12b1f97 Merge pull request #51 from teru1991/codex/add-curve-approximation-and-postprocessing
|\ \ \ \ \  
| * | | | | 364e755 Add deterministic curve approximation and export postprocess pipeline
* | | | | |   5f7aecd Merge pull request #50 from teru1991/codex/remove-ucel-mixed-in-files
|\ \ \ \ \ \  
| |/ / / / /  
|/| | | | |   
| * | | | | f5da151 chore: remove accidentally included UCEL project files
|/ / / / /  
| * / / / 08f0b14 Implement io_json baseline with schema validation and golden roundtrip assets
|/ / / /  
| * / / 6a7f075 Implement io_svg/io_dxf parsers and deterministic IO support integration
|/ / /  
| * / 69edce0 Add io_bridge diycad roundtrip integration and E2E save-reopen flow
|/ /  
| * 6c04c8d Harden support matrix handling and improve SVG/DXF best-effort conversions
|/  
*   7ce9d5f Merge pull request #49 from teru1991/codex/implement-automated-ci-green-loop
|\  
| * 749a2b5 Harden Windows CI by installing Qt and robust Python launcher
| * e146322 Build desktop FFI automatically in build_desktop script
| * 00a064e Add mapping rules doc target required by SSOT lint
| * ac4660f Add unified CI loop scripts and Codex agent guidance
|/  
*   03854bb Merge pull request #48 from teru1991/codex/add-complete-compatibility-policy-ssot-heksyw
|\  
| * 9f1ffc1 Sprint12 PR2: implement core io pipeline and deterministic normalize
* |   cc78882 Merge pull request #46 from teru1991/codex/add-complete-compatibility-policy-ssot
|\ \  
| |/  
|/|   
| * 59bca09 Sprint12 PR1: finalize IO SSOT policies and extend SSOT lint
|/  
*   107c23a Merge pull request #45 from teru1991/codex/implement-dimensions-and-annotations-generation-dz4bhp
|\  
| *   55fb575 Merge branch 'main' into codex/implement-dimensions-and-annotations-generation-dz4bhp
| |\  
| |/  
|/|   
* |   e78d40b Merge pull request #44 from teru1991/codex/implement-dimensions-and-annotations-generation
|\ \  
| * | 1b07306 Add dimension/annotation IR generation and export integration
|/ /  
| * 162c6df Finalize layout collision handling and print-aware export flow
|/  
*   5c4527a Merge pull request #42 from teru1991/codex/add-drawing_style-ssot-and-lint-validation-hiqxgm
|\  
| *   e188807 Merge branch 'main' into codex/add-drawing_style-ssot-and-lint-validation-hiqxgm
| |\  
| |/  
|/|   
* |   6958e05 Merge pull request #41 from teru1991/codex/add-drawing_style-ssot-and-lint-validation-1qoeay
|\ \  
| * | 77eb6e4 Update core/crates/drawing_model/src/validate.rs
| * | 8478d68 Update core/crates/drawing_model/src/validate.rs
| * | 4485c16 Add drawing document model and .diycad salvage integration
* | |   b80fa88 Merge pull request #40 from teru1991/codex/add-drawing_style-ssot-and-lint-validation
|\ \ \  
| |/ /  
|/| |   
| * | 79e50ce Add drawing_style SSOT specs and lint coverage
|/ /  
| * 1298f90 Add RenderIR SVG sheet export pipeline for drawing SSOT
|/  
```

- git show --stat:
```
commit eaeecdab51b401e96c31f343edd4e36a6eab8390
Author: Keiji <teltel1991@gmail.com>
Date:   Mon Mar 2 18:59:41 2026 +0900

    追加、修正

 core/bom/src/lib.rs                                |   2 +-
 core/crates/diagnostics/src/repro.rs               |   2 +-
 core/crates/diycad_common/src/logging.rs           |   4 +-
 core/crates/diycad_format/src/lib.rs               |   2 +-
 core/crates/diycad_geom/tests/circle_arc_ops.rs    |   2 +-
 core/crates/diycad_geom/tests/perf_threshold.rs    |   3 +-
 core/crates/drawing_style/src/annotation/hole.rs   |   2 +-
 core/crates/drawing_style/src/annotation/leader.rs |   4 +-
 core/crates/drawing_style/src/annotation/place.rs  |   4 +-
 core/crates/drawing_style/src/render_svg.rs        |   4 +-
 core/crates/drawing_style/src/sheet.rs             |   2 +-
 core/crates/errors/tests/reason_catalog_lint.rs    |  50 +++---
 .../crates/export_drawing/tests/determinism_svg.rs |   2 +-
 core/crates/io/src/lib.rs                          |   4 +-
 core/crates/io_bridge/src/to_diycad.rs             |   2 +-
 core/crates/io_dxf/src/export.rs                   |   2 +-
 core/crates/io_dxf/src/import.rs                   |  12 +-
 core/crates/io_dxf/src/lib.rs                      |   6 +
 core/crates/io_json/src/import.rs                  |   2 +-
 core/crates/io_json/src/lib.rs                     |  10 +-
 core/crates/io_json/tests/io_roundtrip_golden.rs   |   6 +-
 core/crates/io_support/src/lib.rs                  |   7 -
 core/crates/io_support/tests/io_contract_lint.rs   |  21 +--
 core/crates/io_svg/src/export.rs                   |   7 +-
 core/crates/io_svg/src/lib.rs                      |   6 +
 core/errors/reason_codes.json                      |  11 ++
 core/export/src/pdf_drawing.rs                     |   2 +-
 core/export/src/pdf_tiled.rs                       |   7 +-
 core/export/src/svg.rs                             |   2 +-
 core/export/tests/pdf_invariants.rs                |   5 +-
 core/faces/src/lib.rs                              |   2 +-
 core/i18n/locales/ja-JP.json                       |  12 ++
 core/i18n/src/lib.rs                               |   2 +-
 core/serialize/examples/make_sample.rs             |   2 +-
 core/serialize/tests/spec_ssot_lint.rs             | 177 +++++++--------------
 docs/specs/errors/catalog.json                     |  16 +-
 scripts/ci/run_all.ps1                             |  29 ++--
 37 files changed, 200 insertions(+), 235 deletions(-)
```

- git blame core/crates/io_support/src/lib.rs | head -n 80:
```
6c04c8db (teru1991          2026-03-02 17:12:02 +0900   1) use craftcad_io::model::Units;
6c04c8db (teru1991          2026-03-02 17:12:02 +0900   2) use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000   3) use regex::Regex;
d3345437 (teru1991          2026-03-01 20:16:19 +0900   4) use serde::Deserialize;
d3345437 (teru1991          2026-03-01 20:16:19 +0900   5) use std::collections::BTreeMap;
d3345437 (teru1991          2026-03-01 20:16:19 +0900   6) 
d3345437 (teru1991          2026-03-01 20:16:19 +0900   7) const SUPPORT_MATRIX: &str = include_str!("../../../../docs/specs/io/support_matrix.json");
92d525ad (teru1991          2026-03-01 21:57:16 +0900   8) const MAPPING_RULES: &str = include_str!("../../../../docs/specs/io/mapping_rules.json");
d3345437 (teru1991          2026-03-01 20:16:19 +0900   9) 
d3345437 (teru1991          2026-03-01 20:16:19 +0900  10) #[derive(Debug, Clone, Deserialize)]
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  11) struct SupportMatrixDoc {
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  12)     matrix: Vec<SupportEntry>,
d3345437 (teru1991          2026-03-01 20:16:19 +0900  13) }
d3345437 (teru1991          2026-03-01 20:16:19 +0900  14) 
d3345437 (teru1991          2026-03-01 20:16:19 +0900  15) #[derive(Debug, Clone, Deserialize)]
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  16) struct SupportEntry {
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  17)     format: String,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  18)     direction: String,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  19)     feature: String,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  20)     level: SupportLevel,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  21)     #[serde(default)]
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  22)     action: Option<String>,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  23)     #[serde(default)]
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  24)     reason_codes: Vec<String>,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  25)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  26)     notes: Option<String>,
d3345437 (teru1991          2026-03-01 20:16:19 +0900  27) }
d3345437 (teru1991          2026-03-01 20:16:19 +0900  28) 
92d525ad (teru1991          2026-03-01 21:57:16 +0900  29) #[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
92d525ad (teru1991          2026-03-01 21:57:16 +0900  30) #[serde(rename_all = "snake_case")]
92d525ad (teru1991          2026-03-01 21:57:16 +0900  31) pub enum SupportLevel {
92d525ad (teru1991          2026-03-01 21:57:16 +0900  32)     Supported,
92d525ad (teru1991          2026-03-01 21:57:16 +0900  33)     BestEffort,
92d525ad (teru1991          2026-03-01 21:57:16 +0900  34)     NotSupported,
92d525ad (teru1991          2026-03-01 21:57:16 +0900  35) }
92d525ad (teru1991          2026-03-01 21:57:16 +0900  36) 
92d525ad (teru1991          2026-03-01 21:57:16 +0900  37) #[derive(Debug, Clone, Deserialize)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  38) struct NormalizeDoc {
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  39)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  40)     trim: bool,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  41)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  42)     collapse_whitespace: bool,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  43)     #[serde(default = "default_replace_spaces_with")]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  44)     replace_spaces_with: String,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  45) }
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  46) 
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  47) fn default_replace_spaces_with() -> String {
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  48)     "_".to_string()
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  49) }
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  50) 
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  51) #[derive(Debug, Clone, Deserialize)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  52) struct NamedRulesDoc {
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  53)     default: String,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  54)     max_len: usize,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  55)     forbidden_chars_regex: String,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  56)     normalize: NormalizeDoc,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  57)     aliases: BTreeMap<String, String>,
92d525ad (teru1991          2026-03-01 21:57:16 +0900  58) }
92d525ad (teru1991          2026-03-01 21:57:16 +0900  59) 
92d525ad (teru1991          2026-03-01 21:57:16 +0900  60) #[derive(Debug, Clone, Deserialize)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  61) struct UnitsRulesDoc {
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  62)     supported: Vec<String>,
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  63)     default: String,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  64)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  65)     import_guess_order: Vec<String>,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  66) }
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  67) 
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  68) #[derive(Debug, Clone, Deserialize)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  69) struct ExportRulesDoc {
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  70)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  71)     decimal_places: u8,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  72)     #[serde(default)]
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  73)     force_locale: String,
92d525ad (teru1991          2026-03-01 21:57:16 +0900  74) }
92d525ad (teru1991          2026-03-01 21:57:16 +0900  75) 
92d525ad (teru1991          2026-03-01 21:57:16 +0900  76) #[derive(Debug, Clone, Deserialize)]
6c04c8db (teru1991          2026-03-02 17:12:02 +0900  77) struct MappingRulesDoc {
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  78)     schema_version: u32,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  79)     layer: NamedRulesDoc,
00000000 (Not Committed Yet 2026-03-02 10:07:18 +0000  80)     linetype: NamedRulesDoc,
```

## Tests Run (paste commands + results)
- cd core && cargo fmt --all
- cd core && cargo clippy --workspace --all-targets -- -D warnings
- cd core && cargo test -p craftcad_io_support -p craftcad_io -p craftcad_io_dxf -p craftcad_io_svg

## Self-check (Hard rules)
- [x] Only allowed paths touched
- [x] No deletions (only edits/additions)
- [x] Spec updated first when semantics were clarified
- [x] Added/updated tests (>=1 regression)
- [x] Determinism preserved (stable ordering, no HashMap order dependence introduced)
- [x] No new network/OS dependencies added

### Results
- `cd core && cargo fmt --all`: PASS
- `cd core && cargo clippy --workspace --all-targets -- -D warnings`: FAIL (`core/faces/src/extract.rs` existing clippy lint `manual_is_multiple_of`)
- `cd core && cargo test -p craftcad_io_support -p craftcad_io -p craftcad_io_dxf -p craftcad_io_svg`: PASS
- `./scripts/ci/run_all.sh`: PARTIAL (`rust_fmt` passed, `rust_clippy` failed with same existing lint; run did not complete full pipeline)

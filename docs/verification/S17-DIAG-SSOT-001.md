Verification: S17-DIAG-SSOT-001

## Goal
Sprint17 Diagnostics SSOT を固定し、SSOT lint で機械検証できるようにする。

## Changed files
- docs/specs/diagnostics/README.md (new)
- docs/specs/diagnostics/privacy.md (new)
- docs/specs/diagnostics/retention_policy.md (new)
- docs/specs/diagnostics/repro_template.md (new)
- docs/specs/diagnostics/support_zip.md (new)
- docs/specs/diagnostics/joblog.schema.json (new)
- docs/specs/diagnostics/oplog.schema.json (new)
- core/serialize/tests/spec_ssot_lint.rs (edit)
- (optional) docs/status/trace-index.json (edit; tasks["S17-DIAG-SSOT-001"] only)

## What / Why
- 「再現できる報告」を最短で作るため、JobLog/OpLog/ZIP/保持/プライバシーの契約をSSOT化。
- SSOT lint で docs/specs/diagnostics が壊れたらCIで落とすことで、後続実装の安全地盤を作る。

## Spec alignment checklist
- [x] JobLog schema: header(inputs/seed/eps/round/ordering/limits_profile) を必須化
- [x] OpLog schema: session/actions/seq/params_hash を必須化
- [x] Privacy: パス/個人識別子/生テキスト禁止を明文化
- [x] Support ZIP: 必須ファイルを固定し、同意時のみ任意ファイルを許可
- [x] Retention: keep_days / max_total_bytes / max_items の数値契約を固定

## History evidence (paste command outputs)
- git log -n 30 --oneline -- core/serialize/tests/spec_ssot_lint.rs

```
cb7e1ac S16: integrate perf gate policy and reproducible artifact collection
e0c300a Sprint13: integrate presets/templates references into .diycad document with migration (step6)
b593228 Sprint13: add presets/templates/library SSOT and lint gate (step1)
eaeecda 追加、修正
00a064e Add mapping rules doc target required by SSOT lint
59bca09 Sprint12 PR1: finalize IO SSOT policies and extend SSOT lint
79e50ce Add drawing_style SSOT specs and lint coverage
8786d1e Add drawing style SSOT and deterministic drawing_style crate
d334543 Add SSOT specs and scaffold crates for reason codes/schema/io/golden
```

- git blame -n core/serialize/tests/spec_ssot_lint.rs | head -n 80

```
79e50ce5    1 (teru1991          2026-03-02 06:43:51 +0900    1) use jsonschema::JSONSchema;
79e50ce5    2 (teru1991          2026-03-02 06:43:51 +0900    2) use regex::Regex;
79e50ce5    3 (teru1991          2026-03-02 06:43:51 +0900    3) use serde_json::Value;
79e50ce5    4 (teru1991          2026-03-02 06:43:51 +0900    4) use std::collections::{BTreeSet, HashMap, HashSet};
79e50ce5    5 (teru1991          2026-03-02 06:43:51 +0900    5) use std::fs;
79e50ce5    6 (teru1991          2026-03-02 06:43:51 +0900    6) use std::path::{Path, PathBuf};
d3345437    3 (teru1991          2026-03-01 20:16:19 +0900    7) 
d3345437    4 (teru1991          2026-03-01 20:16:19 +0900    8) #[test]
d3345437    5 (teru1991          2026-03-01 20:16:19 +0900    9) fn reason_catalog_valid_and_links_exist() {
d3345437    6 (teru1991          2026-03-01 20:16:19 +0900   10)     let schema_raw = std::fs::read_to_string("../../docs/specs/errors/catalog.schema.json")
d3345437    7 (teru1991          2026-03-01 20:16:19 +0900   11)         .expect("schema read");
d3345437    8 (teru1991          2026-03-01 20:16:19 +0900   12)     let catalog_raw =
d3345437    9 (teru1991          2026-03-01 20:16:19 +0900   13)         std::fs::read_to_string("../../docs/specs/errors/catalog.json").expect("catalog read");
d3345437   10 (teru1991          2026-03-01 20:16:19 +0900   14) 
d3345437   11 (teru1991          2026-03-01 20:16:19 +0900   15)     let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
d3345437   12 (teru1991          2026-03-01 20:16:19 +0900   16)     let catalog: serde_json::Value = serde_json::from_str(&catalog_raw).expect("catalog json");
d3345437   13 (teru1991          2026-03-01 20:16:19 +0900   17) 
d3345437   14 (teru1991          2026-03-01 20:16:19 +0900   18)     let compiled = jsonschema::JSONSchema::compile(&schema).expect("compile schema");
d3345437   15 (teru1991          2026-03-01 20:16:19 +0900   19)     let result = compiled.validate(&catalog);
d3345437   16 (teru1991          2026-03-01 20:16:19 +0900   20)     if let Err(errors) = result {
d3345437   17 (teru1991          2026-03-01 20:16:19 +0900   21)         let issues: Vec<String> = errors.map(|e| e.to_string()).collect();
d3345437   18 (teru1991          2026-03-01 20:16:19 +0900   22)         panic!("catalog validation failed: {}", issues.join("; "));
d3345437   19 (teru1991          2026-03-01 20:16:19 +0900   23)     }
d3345437   20 (teru1991          2026-03-01 20:16:19 +0900   24) 
d3345437   21 (teru1991          2026-03-01 20:16:19 +0900   25)     let mut uniq = BTreeSet::new();
d3345437   22 (teru1991          2026-03-01 20:16:19 +0900   26)     for item in catalog["items"].as_array().expect("items array") {
d3345437   23 (teru1991          2026-03-01 20:16:19 +0900   27)         let code = item["code"].as_str().expect("code");
d3345437   24 (teru1991          2026-03-01 20:16:19 +0900   28)         assert!(uniq.insert(code.to_string()), "duplicate code: {code}");
d3345437   25 (teru1991          2026-03-01 20:16:19 +0900   29) 
d3345437   26 (teru1991          2026-03-01 20:16:19 +0900   30)         let link = item["doc_link"].as_str().expect("doc_link");
d3345437   27 (teru1991          2026-03-01 20:16:19 +0900   31)         assert!(
d3345437   28 (teru1991          2026-03-01 20:16:19 +0900   32)             Path::new("../..").join(link).exists(),
d3345437   29 (teru1991          2026-03-01 20:16:19 +0900   33)             "missing doc_link target: {link}"
d3345437   30 (teru1991          2026-03-01 20:16:19 +0900   34)         );
d3345437   31 (teru1991          2026-03-01 20:16:19 +0900   35)     }
d3345437   32 (teru1991          2026-03-01 20:16:19 +0900   36) }
d3345437   33 (teru1991          2026-03-01 20:16:19 +0900   37) 
d3345437   34 (teru1991          2026-03-01 20:16:19 +0900   38) #[test]
d3345437   35 (teru1991          2026-03-01 20:16:19 +0900   39) fn io_support_matrix_is_machine_readable() {
d3345437   36 (teru1991          2026-03-01 20:16:19 +0900   40)     let raw =
d3345437   37 (teru1991          2026-03-01 20:16:19 +0900   41)         std::fs::read_to_string("../../docs/specs/io/support_matrix.json").expect("support matrix");
d3345437   38 (teru1991          2026-03-01 20:16:19 +0900   42)     let value: serde_json::Value = serde_json::from_str(&raw).expect("support matrix json");
59bca099   43 (teru1991          2026-03-02 11:35:58 +0900   43)     assert!(value["formats"].is_array());
59bca099   44 (teru1991          2026-03-02 11:35:58 +0900   44)     assert!(value["matrix"].is_array());
d3345437   40 (teru1991          2026-03-01 20:16:19 +0900   45) }
d3345437   41 (teru1991          2026-03-01 20:16:19 +0900   46) 
d3345437   42 (teru1991          2026-03-01 20:16:19 +0900   47) #[test]
d3345437   43 (teru1991          2026-03-01 20:16:19 +0900   48) fn dataset_manifest_references_existing_files() {
d3345437   44 (teru1991          2026-03-01 20:16:19 +0900   49)     let raw =
d3345437   45 (teru1991          2026-03-01 20:16:19 +0900   50)         std::fs::read_to_string("../../tests/datasets/manifest.json").expect("dataset manifest");
d3345437   46 (teru1991          2026-03-01 20:16:19 +0900   51)     let value: serde_json::Value = serde_json::from_str(&raw).expect("dataset manifest json");
d3345437   47 (teru1991          2026-03-01 20:16:19 +0900   52)     for ds in value["datasets"].as_array().expect("datasets") {
cb7e1ac8   53 (teru1991          2026-03-03 22:33:16 +0900   53)         for key in ["inputs", "expected", "expected_outputs"] {
cb7e1ac8   54 (teru1991          2026-03-03 22:33:16 +0900   54)             let Some(entries) = ds.get(key).and_then(|v| v.as_array()) else {
cb7e1ac8   55 (teru1991          2026-03-03 22:33:16 +0900   55)                 continue;
cb7e1ac8   56 (teru1991          2026-03-03 22:33:16 +0900   56)             };
cb7e1ac8   57 (teru1991          2026-03-03 22:33:16 +0900   57)             for p in entries {
cb7e1ac8   58 (teru1991          2026-03-03 22:33:16 +0900   58)                 let rel = p
cb7e1ac8   59 (teru1991          2026-03-03 22:33:16 +0900   59)                     .get("path")
cb7e1ac8   60 (teru1991          2026-03-03 22:33:16 +0900   60)                     .and_then(|v| v.as_str())
cb7e1ac8   61 (teru1991          2026-03-03 22:33:16 +0900   61)                     .or_else(|| p.as_str())
cb7e1ac8   62 (teru1991          2026-03-03 22:33:16 +0900   62)                     .expect("path str");
d3345437   51 (teru1991          2026-03-01 20:16:19 +0900   63)                 assert!(
d3345437   52 (teru1991          2026-03-01 20:16:19 +0900   64)                     Path::new("../..").join(rel).exists(),
d3345437   53 (teru1991          2026-03-01 20:16:19 +0900   65)                     "missing dataset file: {rel}"
d3345437   54 (teru1991          2026-03-01 20:16:19 +0900   66)                 );
d3345437   55 (teru1991          2026-03-01 20:16:19 +0900   67)             }
d3345437   56 (teru1991          2026-03-01 20:16:19 +0900   68)         }
d3345437   57 (teru1991          2026-03-01 20:16:19 +0900   69)     }
d3345437   58 (teru1991          2026-03-01 20:16:19 +0900   70) }
8786d1ed   59 (teru1991          2026-03-01 21:45:33 +0900   71) 
8786d1ed   60 (teru1991          2026-03-01 21:45:33 +0900   72) #[test]
8786d1ed   61 (teru1991          2026-03-01 21:45:33 +0900   73) fn drawing_style_ssot_is_valid_and_named_consistently() {
8786d1ed   62 (teru1991          2026-03-01 21:45:33 +0900   74)     let schema_raw = std::fs::read_to_string("../../docs/specs/drawing/style_ssot.schema.json")
8786d1ed   63 (teru1991          2026-03-01 21:45:33 +0900   75)         .expect("style schema read");
8786d1ed   64 (teru1991          2026-03-01 21:45:33 +0900   76)     let style_raw =
8786d1ed   65 (teru1991          2026-03-01 21:45:33 +0900   77)         std::fs::read_to_string("../../docs/specs/drawing/style_ssot.json").expect("style read");
8786d1ed   66 (teru1991          2026-03-01 21:45:33 +0900   78) 
8786d1ed   67 (teru1991          2026-03-01 21:45:33 +0900   79)     let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("style schema json");
8786d1ed   68 (teru1991          2026-03-01 21:45:33 +0900   80)     let style: serde_json::Value = serde_json::from_str(&style_raw).expect("style json");
```

- rg -n "spec_ssot_lint|schema.json|diagnostics" -S core docs tests

```
docs/specs/drawing/README.md:17:   - `style_ssot.schema.json` に定義を追加
docs/specs/drawing/README.md:19:   - `core/serialize/tests/spec_ssot_lint.rs` で lint と重複検出を更新
docs/specs/system/schema_contract.md:4:- `manifest.schema.json` (`$id`: `https://example.local/schemas/manifest.schema.json`)
docs/specs/system/schema_contract.md:5:- `document.schema.json` (`$id`: `https://example.local/schemas/document.schema.json`)
docs/specs/system/project_format.md:16:- Schema: `core/serialize/schemas/manifest.schema.json`
docs/specs/system/project_format.md:22:- Schema: `core/serialize/schemas/document.schema.json`
docs/specs/project_file/drawing_doc.schema.json:3:  "$id": "https://craftcad.local/specs/project_file/drawing_doc.schema.json",
docs/specs/io/json_internal.schema.json:3:  "$id": "https://craftcad.local/schemas/json_internal_v1.schema.json",
docs/specs/presets/material_preset.schema.json:3:  "$id": "https://craftcad.local/schema/presets/material_preset.schema.json",
docs/specs/presets/presets_bundle.schema.json:3:  "$id": "https://craftcad.local/schema/presets/presets_bundle.schema.json",
docs/specs/presets/presets_bundle.schema.json:11:    "materials": { "type": "array", "items": { "$ref": "material_preset.schema.json" } },
docs/specs/presets/presets_bundle.schema.json:12:    "processes": { "type": "array", "items": { "$ref": "process_preset.schema.json" } },
docs/specs/presets/presets_bundle.schema.json:13:    "outputs": { "type": "array", "items": { "$ref": "output_preset.schema.json" } },
docs/specs/presets/presets_bundle.schema.json:14:    "hardware": { "type": "array", "items": { "$ref": "hardware_preset.schema.json" } }
docs/specs/presets/hardware_preset.schema.json:3:  "$id": "https://craftcad.local/schema/presets/hardware_preset.schema.json",
docs/specs/presets/output_preset.schema.json:3:  "$id": "https://craftcad.local/schema/presets/output_preset.schema.json",
docs/specs/presets/process_preset.schema.json:3:  "$id": "https://craftcad.local/schema/presets/process_preset.schema.json",
docs/specs/testing/datasets_manifest.schema.json:3:  "$id": "craftcad://schemas/testing/datasets_manifest.schema.json",
docs/specs/templates/wizard_template.schema.json:3:  "$id": "https://craftcad.local/schema/templates/wizard_template.schema.json",
docs/specs/templates/README.md:8:- ローカルlint実行コマンド: `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library`
docs/specs/library/tags.schema.json:3:  "$id": "https://craftcad.local/schema/library/tags.schema.json",
docs/specs/schema/versions.md:10:- nest result diagnostics の構造化
docs/specs/schema/diycad/nest_job.schema.json:3:  "$id": "diycad://schema/nest_job.schema.json",
docs/specs/schema/diycad/manifest.schema.json:3:  "$id": "diycad://schema/manifest.schema.json",
docs/specs/schema/diycad/format.md:25:  - /assets/diagnostics/**   (任意。PII禁止。ジョブログ要約等)
docs/specs/schema/diycad/integrity.md:31:  - generate_diagnostics_zip (PII forbidden)
docs/specs/schema/diycad/part.schema.json:3:  "$id": "diycad://schema/part.schema.json",
docs/specs/schema/diycad/document.schema.json:3:  "$id": "diycad://schema/document.schema.json",
docs/specs/drawing_style/print_presets.schema.json:3:  "$id": "https://craftcad.local/specs/drawing_style/print_presets.schema.json",
docs/specs/drawing_style/style_ssot.schema.json:3:  "$id": "https://craftcad.local/specs/drawing_style/style_ssot.schema.json",
docs/specs/drawing_style/sheet_templates.schema.json:3:  "$id": "https://craftcad.local/specs/drawing_style/sheet_templates.schema.json",
tests/diagnostics/joblog_roundtrip.rs:1:use craftcad_diagnostics::{JobLog, JobLogContext, JobStep};
tests/diagnostics/support_zip_redaction.rs:1:use craftcad_diagnostics::SupportZipBuilder;
docs/specs/diagnostics/joblog.schema.json:3:  "$id": "https://craftcad.local/schemas/diagnostics/joblog.schema.json",
docs/specs/diagnostics/repro_template.md:37:- diagnostics_zip: {{zip_name}}
docs/specs/diagnostics/repro_template.md:38:- diagnostics_zip_sha256: {{zip_sha256}}
docs/specs/diagnostics/oplog.schema.json:3:  "$id": "https://craftcad.local/schemas/diagnostics/oplog.schema.json",
docs/specs/diagnostics/README.md:1:Diagnostics SSOT (Sprint17)
docs/specs/diagnostics/README.md:13:- joblog.schema.json: JobLog（正本）— 再現に必要な環境/入力/Reason/タイムライン
docs/specs/diagnostics/README.md:14:- oplog.schema.json: OpLog（最小再生）— Action/Command単位の履歴（UIイベント生ログは禁止）
docs/specs/diagnostics/README.md:21:- “この契約に反するログ/ZIP” はCIで落とします（spec_ssot_lint）。
docs/specs/perf/budgets.schema.json:3:  "$id": "https://craftcad.local/schemas/perf/budgets.schema.json",
docs/specs/errors/catalog.schema.json:3:  "$id": "https://craftcad.local/schemas/reason_catalog.schema.json",
docs/specs/errors/README.md:3:`catalog.schema.json` を唯一の構造定義、`catalog.json` を唯一の値定義として扱います。
docs/runbooks/sprint14_project_survival.md:15:     - `GenerateDiagnosticsZip`（PII禁止）
docs/dev/codex_red_green_loop.md:31:- **Problem:** Stopping at the first failed command loses later diagnostics.
core/crates/diycad_common/src/lib.rs:2:pub mod diagnostics;
core/crates/diycad_common/src/lib.rs:10:pub use diagnostics::{collect_basic_diagnostics, BasicDiagnostics};
core/crates/diycad_common/src/diagnostics.rs:6:pub struct BasicDiagnostics {
core/crates/diycad_common/src/diagnostics.rs:14:pub fn collect_basic_diagnostics() -> BasicDiagnostics {
core/crates/diycad_common/src/diagnostics.rs:20:    BasicDiagnostics {
core/crates/diycad_common/src/diagnostics.rs:34:    fn diagnostics_contains_core_fields() {
core/crates/diycad_common/src/diagnostics.rs:35:        let diagnostics = collect_basic_diagnostics();
core/crates/diycad_common/src/diagnostics.rs:36:        assert!(!diagnostics.os.is_empty());
core/crates/diycad_common/src/diagnostics.rs:37:        assert!(!diagnostics.arch.is_empty());
core/crates/diycad_common/src/diagnostics.rs:38:        assert!(!diagnostics.app_version.is_empty());
core/crates/diycad_common/src/diagnostics.rs:39:        assert!(!diagnostics.time.is_empty());
docs/verification/S13-WIZARDS-IMPL-005.md:45:- `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library` (in `core/`) → pass
docs/verification/S12-IO-SSOT-001.md:173: core/crates/diagnostics/src/repro.rs               |   2 +-
docs/verification/S12-IO-SSOT-001.md:206: core/serialize/tests/spec_ssot_lint.rs             | 177 +++++++--------------
docs/verification/S13-PRESETS-SSOT-LINT-001.md:4:Sprint13 PR1相当：Presets/Template/Library/DeterminismのSSOT追加と、spec_ssot_lintでのCIゲート化。
docs/verification/S13-PRESETS-SSOT-LINT-001.md:10:core/serialize/tests/spec_ssot_lint.rs
docs/verification/S13-PRESETS-SSOT-LINT-001.md:16: M core/serialize/tests/spec_ssot_lint.rs
docs/verification/S13-PRESETS-SSOT-LINT-001.md:72:- `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library` =>
docs/verification/S13-PRESETS-SSOT-LINT-001.md:99:     Running tests/spec_ssot_lint.rs (target/debug/deps/spec_ssot_lint-8196b5f771b362dc)
docs/verification/S13-PRESETS-SSOT-LINT-001.md:102:test spec_ssot_lint_presets_templates_library ... ok
docs/verification/S13-PRESETS-SSOT-LINT-001.md:178:- tags.schema.jsonはpolicy objectとして運用（Step3でreader実装）
docs/verification/S13-PRESETS-SSOT-LINT-001.md:190: core/serialize/tests/spec_ssot_lint.rs             | 298 +++++++++++++++++++++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:194: docs/specs/library/tags.schema.json                |  37 +++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:197: docs/specs/presets/hardware_preset.schema.json     |  43 +++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:199: docs/specs/presets/material_preset.schema.json     |  30 +++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:200: docs/specs/presets/output_preset.schema.json       |  30 +++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:201: docs/specs/presets/presets_bundle.schema.json      |  16 ++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:202: docs/specs/presets/process_preset.schema.json      |  33 +++
docs/verification/S13-PRESETS-SSOT-LINT-001.md:207: docs/specs/templates/wizard_template.schema.json   |  77 ++++++
docs/verification/S16-PERF-DATASET-SMOKE-006.md:86:- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint` => PASS
docs/verification/S16-PERF-DATASET-SMOKE-006.md:96:  - `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` failed as expected
docs/verification/S13-DIYCAD-ASSETS-INTEGRATION-006.md:7:- `.diycad` schema SSOT path: `docs/specs/schema/diycad/document.schema.json`
docs/verification/S13-DIYCAD-ASSETS-INTEGRATION-006.md:11:- `docs/specs/schema/diycad/document.schema.json`
docs/verification/S13-DIYCAD-ASSETS-INTEGRATION-006.md:12:- `core/serialize/schemas/document.schema.json`
docs/verification/S13-WIZARDS-ENGINE-004.md:25:- `wizard_template.schema.json` によるテンプレ検証。
core/crates/diycad_format/src/integrity.rs:87:        salvage.push(SalvageActionHint::GenerateDiagnosticsZip);
docs/verification/S15-STEP1.md:35:- `docs/specs/testing/datasets_manifest.schema.json`
core/crates/diycad_format/src/types.rs:123:    GenerateDiagnosticsZip,
docs/verification/S16-PERF-SSOT-001.md:7:- Added: docs/specs/perf/budgets.schema.json
docs/verification/S16-PERF-SSOT-001.md:14:- Updated: core/serialize/tests/spec_ssot_lint.rs
docs/verification/S16-PERF-SSOT-001.md:35:  modified:   core/serialize/tests/spec_ssot_lint.rs
docs/verification/S16-PERF-SSOT-001.md:37:  modified:   docs/specs/perf/budgets.schema.json
docs/verification/S16-PERF-SSOT-001.md:77:- `cargo test -p craftcad_serialize --test spec_ssot_lint` => PASS (10 passed)
docs/verification/S16-PERF-SSOT-001.md:81:    - `cargo test -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` failed with:
docs/verification/S16-PERF-SSOT-001.md:84:    - corrupted `budgets.schema.json`
docs/verification/S16-PERF-SSOT-001.md:86:      - `budgets.schema.json must be valid JSON`
docs/verification/S13-LIBRARY-CRATE-003.md:37:- `docs/specs/library/tags.schema.json`: policy準拠（normalize/forbidden/max_len）。
core/crates/diycad_format/src/open.rs:388:        SalvageActionHint::GenerateDiagnosticsZip,
docs/verification/SPRINT14-STEP1.md:8:- docs/specs/schema/diycad/{manifest.schema.json,document.schema.json,part.schema.json,nest_job.schema.json}
docs/verification/S16-PERF-GATE-INTEGRATION-008.md:11:- Updated: core/serialize/tests/spec_ssot_lint.rs (policy sanity check)
docs/verification/S16-PERF-GATE-INTEGRATION-008.md:23:  modified:   core/serialize/tests/spec_ssot_lint.rs
docs/verification/S16-PERF-GATE-INTEGRATION-008.md:43:- `cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_perf_budgets_json_is_valid_and_consistent` => PASS
docs/verification/S13-QUALITY-GATES-E2E-007.md:26:- `cargo test -p craftcad_serialize --test spec_ssot_lint`
docs/verification/S13-QUALITY-GATES-E2E-007.md:27:  - PASS (`spec_ssot_lint_presets_templates_library` を含む9件成功)
docs/status/trace-index.json:69:        "tests": "cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library: pass; cargo test (core workspace): fail at existing compat_report_golden ordering mismatch"
docs/status/trace-index.json:109:        "docs/specs/schema/diycad/document.schema.json",
docs/status/trace-index.json:134:        "tests": "cargo test -p craftcad_serialize --test spec_ssot_lint: pass; cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export: pass; cargo test -p craftcad_wizards --test wizard_determinism: pass; cargo test -p craftcad_wizards --test presets_templates_compat: pass; cargo test --workspace --all-targets: fails at existing craftcad_io_bridge compat_matrix_golden ordering mismatch"
docs/status/trace-index.json:241:        "docs/specs/testing/datasets_manifest.schema.json",
docs/status/trace-index.json:302:        "docs/specs/perf/budgets.schema.json",
docs/status/trace-index.json:309:        "core/serialize/tests/spec_ssot_lint.rs",
docs/status/trace-index.json:398:        "core/serialize/tests/spec_ssot_lint.rs",
core/crates/diycad_ffi/src/lib.rs:2:use diycad_common::{collect_basic_diagnostics, init_logging, log_info};
core/crates/diycad_ffi/src/lib.rs:19:        let diagnostics = collect_basic_diagnostics();
core/crates/diycad_ffi/src/lib.rs:20:        CString::new(diagnostics.app_version).expect("version string must not contain NUL")
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:88:    assert!(!ws.is_empty(), "eviction should be recorded for diagnostics");
core/crates/cache/src/reasons.rs:7:/// Stable reason code for cache diagnostics.
core/crates/io_json/src/schema.rs:7:const SCHEMA_PATH: &str = "docs/specs/io/json_internal.schema.json";
core/crates/io_json/src/schema.rs:20:        .unwrap_or_else(|e| panic!("invalid schema json {}: {}", p.display(), e))
core/crates/diagnostics/Cargo.toml:2:name = "craftcad_diagnostics"
core/crates/wizards/src/template.rs:60:        let schema_path = dir.join("wizard_template.schema.json");
core/crates/diycad_document/tests/schema_validation.rs:4:fn compile_schema(schema_json: &Value) -> JSONSchema {
core/crates/diycad_document/tests/schema_validation.rs:6:        .compile(schema_json)
core/crates/diycad_document/tests/schema_validation.rs:13:        "../../../../docs/specs/project_file/drawing_doc.schema.json"
core/Cargo.toml:29:  "crates/diagnostics",
core/export/schemas/export_options.schema.json:3:  "$id": "https://example.local/schemas/export_options.schema.json",
core/export/tests/export_contract_lint.rs:7:    let raw = std::fs::read_to_string("schemas/export_options.schema.json")
core/export/tests/export_contract_lint.rs:8:        .or_else(|_| std::fs::read_to_string("../export/schemas/export_options.schema.json"))
core/crates/ssot_lint/src/lib.rs:104:    let manifest_schema = base.join("manifest.schema.json");
core/crates/ssot_lint/src/lib.rs:105:    let document_schema = base.join("document.schema.json");
core/crates/ssot_lint/src/lib.rs:106:    let part_schema = base.join("part.schema.json");
core/crates/ssot_lint/src/lib.rs:107:    let nest_job_schema = base.join("nest_job.schema.json");
core/crates/ssot_lint/src/lib.rs:185:    // manifest.schema.json required keys and minimum schema_version alignment with latest
core/crates/ssot_lint/src/lib.rs:208:          message: format!("manifest.schema.json schema_version.minimum ({}) must be <= latest_schema_version ({})", min, latest),
core/crates/ssot_lint/src/lib.rs:240:        std::fs::write(dir.join("manifest.schema.json"), r#"{"type":"object","required":["schema_version","app_version","created_at","updated_at","unit","entrypoints"],"properties":{"schema_version":{"minimum":1}}}"#).unwrap();
core/crates/ssot_lint/src/lib.rs:241:        std::fs::write(dir.join("document.schema.json"), r#"{"type":"object","required":["id","name","unit","entities","parts_index","nest_jobs_index"]}"#).unwrap();
core/crates/ssot_lint/src/lib.rs:243:            dir.join("part.schema.json"),
core/crates/ssot_lint/src/lib.rs:248:            dir.join("nest_job.schema.json"),
core/serialize/schemas/manifest.schema.json:3:  "$id": "https://example.local/schemas/manifest.schema.json",
core/serialize/schemas/document.schema.json:3:  "$id": "https://example.local/schemas/document.schema.json",
core/crates/library/src/tags.rs:45:        .join("tags.schema.json");
core/serialize/tests/spec_ssot_lint.rs:10:    let schema_raw = std::fs::read_to_string("../../docs/specs/errors/catalog.schema.json")
core/serialize/tests/spec_ssot_lint.rs:15:    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
core/serialize/tests/spec_ssot_lint.rs:74:    let schema_raw = std::fs::read_to_string("../../docs/specs/drawing/style_ssot.schema.json")
core/serialize/tests/spec_ssot_lint.rs:79:    let schema: serde_json::Value = serde_json::from_str(&schema_raw).expect("style schema json");
core/serialize/tests/spec_ssot_lint.rs:137:    let schema_json = read_json(schema_path);
core/serialize/tests/spec_ssot_lint.rs:139:        .compile(&schema_json)
core/serialize/tests/spec_ssot_lint.rs:191:    let style_schema_path = base.join("style_ssot.schema.json");
core/serialize/tests/spec_ssot_lint.rs:194:    let sheet_schema_path = base.join("sheet_templates.schema.json");
core/serialize/tests/spec_ssot_lint.rs:197:    let print_schema_path = base.join("print_presets.schema.json");
core/serialize/tests/spec_ssot_lint.rs:679:    let mut schema_json = read_json(schema_path);
core/serialize/tests/spec_ssot_lint.rs:680:    resolve_local_refs_s13(&mut schema_json, schema_root);
core/serialize/tests/spec_ssot_lint.rs:683:        .compile(&schema_json)
core/serialize/tests/spec_ssot_lint.rs:729:        panic!("tags.schema.json: schema_version must be >=1");
core/serialize/tests/spec_ssot_lint.rs:737:        panic!("tags.schema.json: max_len must be 32 (got {})", max_len);
core/serialize/tests/spec_ssot_lint.rs:743:        .unwrap_or_else(|| panic!("tags.schema.json: normalize missing"));
core/serialize/tests/spec_ssot_lint.rs:747:            panic!("tags.schema.json: normalize.{} must be true", k);
core/serialize/tests/spec_ssot_lint.rs:758:        .unwrap_or_else(|| panic!("tags.schema.json: forbidden_patterns missing"));
core/serialize/tests/spec_ssot_lint.rs:766:            panic!("tags.schema.json: forbidden_patterns must include '{}'", m);
core/serialize/tests/spec_ssot_lint.rs:797:fn spec_ssot_lint_presets_templates_library() {
core/serialize/tests/spec_ssot_lint.rs:804:    let material_schema = presets_dir.join("material_preset.schema.json");
core/serialize/tests/spec_ssot_lint.rs:805:    let process_schema = presets_dir.join("process_preset.schema.json");
core/serialize/tests/spec_ssot_lint.rs:806:    let output_schema = presets_dir.join("output_preset.schema.json");
core/serialize/tests/spec_ssot_lint.rs:807:    let hardware_schema = presets_dir.join("hardware_preset.schema.json");
core/serialize/tests/spec_ssot_lint.rs:808:    let bundle_schema = presets_dir.join("presets_bundle.schema.json");
core/serialize/tests/spec_ssot_lint.rs:809:    let template_schema = templates_dir.join("wizard_template.schema.json");
core/serialize/tests/spec_ssot_lint.rs:810:    let tags_schema = library_dir.join("tags.schema.json");
core/serialize/tests/spec_ssot_lint.rs:997:    let schema_path = root.join("docs/specs/perf/budgets.schema.json");
core/serialize/tests/spec_ssot_lint.rs:999:    assert!(schema_path.exists(), "Missing budgets.schema.json: {}", schema_path.display());
core/serialize/tests/spec_ssot_lint.rs:1003:        std::fs::read_to_string(&schema_path).expect("Failed to read budgets.schema.json");
core/serialize/tests/spec_ssot_lint.rs:1007:    let schema_json: serde_json::Value =
core/serialize/tests/spec_ssot_lint.rs:1008:        serde_json::from_str(&schema_text).expect("budgets.schema.json must be valid JSON");
core/serialize/tests/spec_ssot_lint.rs:1010:    let compiled = jsonschema::JSONSchema::compile(&schema_json)
core/serialize/tests/spec_ssot_lint.rs:1011:        .expect("Failed to compile budgets.schema.json");
core/serialize/tests/spec_ssot_lint.rs:1117:fn ssot_diagnostics_contracts_exist_and_valid() {
core/serialize/tests/spec_ssot_lint.rs:1119:    let dir = root.join("docs").join("specs").join("diagnostics");
core/serialize/tests/spec_ssot_lint.rs:1122:        "missing diagnostics ssot dir: {}",
core/serialize/tests/spec_ssot_lint.rs:1132:        "joblog.schema.json",
core/serialize/tests/spec_ssot_lint.rs:1133:        "oplog.schema.json",
core/serialize/tests/spec_ssot_lint.rs:1139:            "missing required diagnostics spec file: {}",
core/serialize/tests/spec_ssot_lint.rs:1144:    let joblog_schema = read_json(&dir.join("joblog.schema.json"));
core/serialize/tests/spec_ssot_lint.rs:1145:    let oplog_schema = read_json(&dir.join("oplog.schema.json"));
core/serialize/tests/spec_ssot_lint.rs:1148:        .unwrap_or_else(|e| panic!("joblog.schema.json is not a valid JSON Schema: {e:?}"));
core/serialize/tests/spec_ssot_lint.rs:1156:        .unwrap_or_else(|e| panic!("oplog.schema.json is not a valid JSON Schema: {e:?}"));
core/serialize/tests/schema_lint.rs:3:    let doc_raw = std::fs::read_to_string("schemas/document.schema.json").expect("read doc schema");
core/serialize/tests/schema_lint.rs:5:        std::fs::read_to_string("schemas/manifest.schema.json").expect("read manifest schema");
core/serialize/src/lib.rs:497:    let schema = compile_schema(include_str!("../schemas/manifest.schema.json"))?;
core/serialize/src/lib.rs:560:    let schema = compile_schema(include_str!("../schemas/document.schema.json"))?;
```

## Tests / Commands executed
- [x] cargo test --manifest-path core/Cargo.toml -p craftcad_serialize --test spec_ssot_lint ssot_diagnostics_contracts_exist_and_valid
- [ ] cargo test (workspace)  # if reasonable

## Safety / Allowlist self-check
- [x] Edited paths are within allowlist only
- [x] No deletions (only new files + minimal edits)
- [x] No PII introduced in specs (privacy.md confirmed)

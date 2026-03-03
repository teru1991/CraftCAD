Verification: S17-DIAG-CORE-002

## Goal
- diagnostics core crate を追加し、JobLog/OpLog/ReasonSummary/Repro を決定性・PII排除・limits耐性で完成。

## Changed files
- core/crates/diagnostics/Cargo.toml (new)
- core/crates/diagnostics/src/lib.rs (new)
- core/crates/diagnostics/src/security_iface.rs (new)
- core/crates/diagnostics/src/reasons.rs (new)
- core/crates/diagnostics/src/joblog.rs (new)
- core/crates/diagnostics/src/oplog.rs (new)
- core/crates/diagnostics/src/reason_summary.rs (new)
- core/crates/diagnostics/src/repro.rs (new)
- core/crates/diagnostics/src/ssot_fingerprint.rs (new; skeleton)
- core/crates/diagnostics/src/retention.rs (new; skeleton)
- core/crates/diagnostics/tests/basic.rs (new)
- (optional) Cargo.toml / Cargo.lock (workspace wiring)
- (optional) docs/status/trace-index.json (tasks["S17-DIAG-CORE-002"] only)

## What / Why
- Sprint17の中核「再現情報の正本(JobLog)」「最小再生(OpLog)」「原因要約(ReasonSummary)」「Issue貼り付け(Repro)」を製品品質で実装する土台。
- Sprint18でredaction本実装に差し替え可能な security_iface を用意し、Step2でも安全側に倒す。

## Determinism & Privacy
- JSON出力はBTreeMap/BTreeSet + stable sort で順序固定。
- params_redacted は Redactor 経由、かつサイズが大きすぎれば {"_truncated":true} に置換。
- 生パス/生テキストは保持しない前提（privacy SSOT準拠）。

## History evidence (paste outputs)
- git log -n 25 --oneline

```
99c259b S17: add diagnostics SSOT specs and enforce via spec_ssot_lint
3946aec Merge pull request #85 from teru1991/codex/implement-ssot-contracts-for-performance-e4vjxw
5f15c15 Merge branch 'main' into codex/implement-ssot-contracts-for-performance-e4vjxw
cb7e1ac S16: integrate perf gate policy and reproducible artifact collection
c74aa37 Merge pull request #84 from teru1991/codex/implement-ssot-contracts-for-performance-0gqdms
7174be0 Merge branch 'main' into codex/implement-ssot-contracts-for-performance-0gqdms
6afcf25 S16: add perf bench executables and report artifact tooling
81bed05 Merge pull request #83 from teru1991/codex/implement-ssot-contracts-for-performance-ndqnlo
a62a5e2 Merge branch 'main' into codex/implement-ssot-contracts-for-performance-ndqnlo
9415d3b S16: add heavy dataset perf smoke gate with report artifacts
be27f4c Merge pull request #82 from teru1991/codex/implement-ssot-contracts-for-performance-697wmk
eac35ee Merge branch 'main' into codex/implement-ssot-contracts-for-performance-697wmk
748e221 S16: add deterministic cache crate with bounded LRU store
625618e Merge pull request #81 from teru1991/codex/implement-ssot-contracts-for-performance-8pguly
8e7d82b Merge branch 'main' into codex/implement-ssot-contracts-for-performance-8pguly
c48958b S16: implement deterministic jobs queue core with contract tests
8f16772 Merge pull request #80 from teru1991/codex/implement-ssot-contracts-for-performance
f28e6e2 S16: fix perf SSOT budgets contract and lint gate
298cb0b Merge pull request #79 from teru1991/codex/expand-golden-datasets-for-step-3-thjhms
609d9fe Merge branch 'main' into codex/expand-golden-datasets-for-step-3-thjhms
70a5816 S15-STEP5: add 10x determinism gate with shared dataset runner
2d283fe Merge pull request #78 from teru1991/codex/expand-golden-datasets-for-step-3-qvdofx
a42e8f1 Merge branch 'main' into codex/expand-golden-datasets-for-step-3-qvdofx
98154eb S15-STEP4: add binary-free N-2 compat assets and harness
8cb360f Merge pull request #77 from teru1991/codex/expand-golden-datasets-for-step-3
```

- rg -n "ReasonCatalog|ReasonCode|redaction|limits|consent|diagnostics" -S core core/Cargo.toml

```
core/Cargo.toml:29:  "crates/diagnostics",
core/crates/io_bridge/src/from_diycad.rs:3:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_bridge/src/from_diycad.rs:46:            ReasonCode::LOAD_DIYCAD_READ_FAILED,
core/crates/io_bridge/src/from_diycad.rs:68:                        ReasonCode::LOAD_PART_EXPORT_FAILED,
core/crates/io_bridge/src/from_diycad.rs:133:                ReasonCode::LOAD_DIYCAD_EMPTY,
core/crates/io_bridge/src/to_diycad.rs:3:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_bridge/src/to_diycad.rs:74:                                        ReasonCode::SAVE_GEOM_DROPPED,
core/crates/io_bridge/src/to_diycad.rs:83:                                    ReasonCode::SAVE_PART_IMPORT_FAILED,
core/crates/io_bridge/src/to_diycad.rs:103:                            ReasonCode::SAVE_PART_IMPORT_FAILED,
core/crates/io_bridge/src/to_diycad.rs:114:                        ReasonCode::SAVE_TEXT_BEST_EFFORT,
core/crates/io_bridge/src/to_diycad.rs:129:                        ReasonCode::SAVE_PART_IMPORT_FAILED,
core/crates/io_bridge/src/to_diycad.rs:144:            ReasonCode::SAVE_DIYCAD_WRITE_FAILED,
core/crates/wizards/src/template.rs:2:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/template.rs:25:        WizardReason::new(WizardReasonCode::WizardIoError, format!("read failed: {e}"))
core/crates/wizards/src/template.rs:30:            WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:44:                WizardReasonCode::WizardTemplateSchemaInvalid,
core/crates/wizards/src/template.rs:78:                WizardReasonCode::WizardTemplateSchemaInvalid,
core/crates/wizards/src/template.rs:85:                WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:106:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:112:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/template.rs:125:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:131:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/template.rs:144:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:150:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/template.rs:163:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/template.rs:169:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/engine/eval.rs:3:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/engine/eval.rs:12:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:21:            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:31:            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:57:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:72:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/eval.rs:80:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:99:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:116:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:130:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:140:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:150:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:160:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:185:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:200:                        WizardReason::new(WizardReasonCode::WizardDslInvalid, "text_key missing")
core/crates/wizards/src/engine/eval.rs:211:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval.rs:235:                    WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:1:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/engine/eval_expr.rs:30:            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:75:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:90:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:98:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:144:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:152:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:161:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:182:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:204:                        WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:214:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:222:            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/engine/eval_expr.rs:232:        WizardReason::new(WizardReasonCode::WizardDslInvalid, "expr stack underflow")
core/crates/wizards/src/engine/eval_expr.rs:235:        WizardReason::new(WizardReasonCode::WizardDslInvalid, "expr stack underflow")
core/crates/wizards/src/engine/validate.rs:2:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/engine/validate.rs:37:                WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:51:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:58:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/engine/validate.rs:64:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/engine/validate.rs:70:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:80:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:87:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/engine/validate.rs:93:                    WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/engine/validate.rs:99:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:109:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:119:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:126:                    WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/engine/validate.rs:134:            WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/engine/validate.rs:156:            WizardReasonCode::WizardDeterminismError,
core/crates/wizards/src/determinism.rs:1:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/determinism.rs:28:                WizardReasonCode::WizardDeterminismError,
core/crates/wizards/src/leather_pouch.rs:3:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/leather_pouch.rs:8:        .ok_or_else(|| WizardReason::new(WizardReasonCode::WizardDslInvalid, "expected number"))
core/crates/wizards/src/leather_pouch.rs:36:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing w_mm")
core/crates/wizards/src/leather_pouch.rs:39:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing h_mm")
core/crates/wizards/src/leather_pouch.rs:51:                        WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/leather_pouch.rs:57:                        WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/leather_pouch.rs:72:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/leather_pouch.rs:87:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/leather_pouch.rs:105:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/leather_pouch.rs:123:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/lib.rs:15:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/lib.rs:55:                WizardReasonCode::WizardIoError,
core/crates/wizards/src/lib.rs:75:                WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/lib.rs:98:                    WizardReasonCode::WizardDeterminismError,
core/crates/wizards/src/lib.rs:164:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/lib.rs:175:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/lib.rs:186:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/lib.rs:198:            WizardReason::new(WizardReasonCode::WizardIoError, format!("read failed: {e}"))
core/crates/wizards/src/lib.rs:203:                WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/lib.rs:215:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/lib.rs:225:                    WizardReasonCode::WizardDepMissingPreset,
core/crates/wizards/src/lib.rs:291:                WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/box.rs:3:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/box.rs:8:        .ok_or_else(|| WizardReason::new(WizardReasonCode::WizardDslInvalid, "expected number"))
core/crates/wizards/src/box.rs:34:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing w_mm")
core/crates/wizards/src/box.rs:37:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing h_mm")
core/crates/wizards/src/parts/validate.rs:2:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/parts/validate.rs:11:            WizardReasonCode::WizardTemplateInvalid,
core/crates/wizards/src/parts/validate.rs:17:            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/parts/validate.rs:25:                WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/parts/validate.rs:31:                WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/parts/validate.rs:40:                        WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/parts/validate.rs:57:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/parts/validate.rs:64:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/parts/validate.rs:82:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/parts/validate.rs:88:                            WizardReasonCode::WizardInputInvalid,
core/crates/wizards/src/reasons.rs:4:pub enum WizardReasonCode {
core/crates/wizards/src/reasons.rs:16:    pub code: WizardReasonCode,
core/crates/wizards/src/reasons.rs:23:    pub fn new(code: WizardReasonCode, message: impl Into<String>) -> Self {
core/crates/wizards/src/shelf.rs:3:use crate::reasons::{WizardReason, WizardReasonCode};
core/crates/wizards/src/shelf.rs:8:        .ok_or_else(|| WizardReason::new(WizardReasonCode::WizardDslInvalid, "expected number"))
core/crates/wizards/src/shelf.rs:36:            WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing w_mm")
core/crates/wizards/src/shelf.rs:39:            WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing h_mm")
core/crates/wizards/src/shelf.rs:60:                        WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing part_name")
core/crates/wizards/src/shelf.rs:64:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing diameter_mm")
core/crates/wizards/src/shelf.rs:67:                    WizardReason::new(WizardReasonCode::WizardDslInvalid, "missing offset_mm")
core/crates/wizards/src/shelf.rs:80:                            WizardReasonCode::WizardDslInvalid,
core/crates/wizards/src/shelf.rs:97:                            WizardReasonCode::WizardDslInvalid,
core/crates/io_support/tests/support_matrix_reason_mapping.rs:1:use craftcad_io::reasons::ReasonCode;
core/crates/io_support/tests/support_matrix_reason_mapping.rs:9:    assert!(dxf_text.contains(&ReasonCode::IO_TEXT_FALLBACK_FONT));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:10:    assert!(dxf_text.contains(&ReasonCode::IO_FALLBACK_024));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:13:    assert!(dxf_spline.contains(&ReasonCode::IO_CURVE_APPROX_APPLIED));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:14:    assert!(dxf_spline.contains(&ReasonCode::IO_UNSUPPORTED_ENTITY_DXF_SPLINE));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:17:    assert!(dxf_hatch.contains(&ReasonCode::IO_HATCH_SIMPLIFIED));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:20:    assert!(svg_text.contains(&ReasonCode::IO_TEXT_FALLBACK_FONT));
core/crates/io_support/tests/support_matrix_reason_mapping.rs:23:    assert!(svg_ext.contains(&ReasonCode::IO_IMAGE_REFERENCE_DROPPED));
core/crates/io_support/src/lib.rs:2:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_support/src/lib.rs:145:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:164:    pub fn reasons(&self, format: &str, feature: &str, direction: &str) -> Vec<ReasonCode> {
core/crates/io_support/src/lib.rs:169:            .unwrap_or_else(|| vec![ReasonCode::IO_SUPPORT_MATRIX_FEATURE_MISSING])
core/crates/io_support/src/lib.rs:184:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:193:                ReasonCode::IO_JSON_SCHEMA_UNSUPPORTED_VERSION,
core/crates/io_support/src/lib.rs:202:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:211:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:220:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "max_len must be > 0").fatal(),
core/crates/io_support/src/lib.rs:229:                    ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:244:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_support/src/lib.rs:324:fn map_reason_code(s: &str) -> ReasonCode {
core/crates/io_support/src/lib.rs:326:        "IO_CURVE_APPROX_APPLIED" => ReasonCode::IO_CURVE_APPROX_APPLIED,
core/crates/io_support/src/lib.rs:327:        "IO_TEXT_FALLBACK_FONT" => ReasonCode::IO_TEXT_FALLBACK_FONT,
core/crates/io_support/src/lib.rs:328:        "IO_FALLBACK_024" => ReasonCode::IO_FALLBACK_024,
core/crates/io_support/src/lib.rs:329:        "IO_UNSUPPORTED_ENTITY_DXF_SPLINE" => ReasonCode::IO_UNSUPPORTED_ENTITY_DXF_SPLINE,
core/crates/io_support/src/lib.rs:330:        "IO_HATCH_SIMPLIFIED" => ReasonCode::IO_HATCH_SIMPLIFIED,
core/crates/io_support/src/lib.rs:331:        "IO_IMAGE_REFERENCE_DROPPED" => ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
core/crates/io_support/src/lib.rs:332:        _ => ReasonCode::IO_SUPPORT_MATRIX_FEATURE_MISSING,
core/crates/io_svg/tests/svg_path_parser.rs:3:use craftcad_io::reasons::ReasonCode;
core/crates/io_svg/tests/svg_path_parser.rs:67:            .any(|w| w.reason == ReasonCode::IO_CURVE_APPROX_APPLIED),
core/crates/io_svg/tests/svg_limits_external_ref.rs:2:use craftcad_io::reasons::ReasonCode;
core/crates/io_svg/tests/svg_limits_external_ref.rs:7:fn svg_respects_limits_max_entities() {
core/crates/io_svg/tests/svg_limits_external_ref.rs:19:    opts.limits.max_entities = 5;
core/crates/io_svg/tests/svg_limits_external_ref.rs:22:    assert_eq!(err.reason, ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED);
core/crates/io_svg/tests/svg_limits_external_ref.rs:36:            .any(|w| w.reason == ReasonCode::IO_IMAGE_REFERENCE_DROPPED),
core/crates/io_dxf/src/import.rs:5:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_dxf/src/import.rs:160:                            ReasonCode::IO_UNIT_GUESSED,
core/crates/io_dxf/src/import.rs:173:                ReasonCode::IO_UNIT_GUESSED,
core/crates/io_dxf/src/import.rs:354:                            ReasonCode::IO_DXF_SPLINE_CONVERTED,
core/crates/io_dxf/src/import.rs:365:                            ReasonCode::IO_DXF_SPLINE_CONVERTED,
core/crates/io_dxf/src/import.rs:386:                        ReasonCode::IO_DXF_ENTITY_UNKNOWN_DROPPED,
core/crates/io_svg/src/import.rs:7:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_svg/src/import.rs:178:                ReasonCode::IO_CURVE_APPROX_APPLIED,
core/crates/io_svg/src/import.rs:383:                ReasonCode::IO_UNIT_GUESSED,
core/crates/io_dxf/src/parse.rs:2:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_dxf/src/parse.rs:32:            ReasonCode::IO_PARSE_DXF_MALFORMED,
core/crates/io_dxf/src/parse.rs:40:        .limits
core/crates/io_dxf/src/parse.rs:45:        .limits
core/crates/io_dxf/src/parse.rs:59:                AppError::new(ReasonCode::IO_PARSE_DXF_MALFORMED, "odd number of lines").fatal(),
core/crates/io_dxf/src/parse.rs:66:                ReasonCode::IO_DXF_LIMIT_LINES_EXCEEDED,
core/crates/io_dxf/src/parse.rs:75:                ReasonCode::IO_DXF_LIMIT_GROUPS_EXCEEDED,
core/crates/io_dxf/src/parse.rs:84:                ReasonCode::IO_DXF_LIMIT_STRING_EXCEEDED,
core/crates/io_dxf/src/parse.rs:93:            AppError::new(ReasonCode::IO_PARSE_DXF_MALFORMED, "invalid group code")
core/crates/io_svg/src/parse.rs:2:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_svg/src/parse.rs:33:    if bytes.len() > opts.limits.max_bytes {
core/crates/io_svg/src/parse.rs:34:        return Err(AppError::new(ReasonCode::IO_LIMIT_BYTES_EXCEEDED, "input too large").fatal());
core/crates/io_svg/src/parse.rs:40:    let max_nodes = opts.limits.max_entities.max(1);
core/crates/io_svg/src/parse.rs:41:    let max_depth = opts.limits.max_depth.max(1);
core/crates/io_svg/src/parse.rs:54:                        ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED,
core/crates/io_svg/src/parse.rs:63:                        ReasonCode::IO_SVG_LIMIT_DEPTH_EXCEEDED,
core/crates/io_svg/src/parse.rs:80:                                ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/parse.rs:91:                                ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
core/crates/io_svg/src/parse.rs:115:                        ReasonCode::IO_SVG_LIMIT_NODES_EXCEEDED,
core/crates/io_svg/src/parse.rs:132:                                ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/parse.rs:143:                                ReasonCode::IO_IMAGE_REFERENCE_DROPPED,
core/crates/io_svg/src/parse.rs:174:                    AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "unbalanced svg tags").fatal()
core/crates/io_svg/src/parse.rs:192:                                ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/parse.rs:209:                    AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "malformed svg")
core/crates/io_svg/src/parse.rs:220:    Err(AppError::new(ReasonCode::IO_PARSE_SVG_MALFORMED, "unexpected eof").fatal())
core/crates/io_svg/src/pathdata.rs:3:use craftcad_io::reasons::{AppError, ReasonCode};
core/crates/io_svg/src/pathdata.rs:225:                            ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/pathdata.rs:511:                        ReasonCode::IO_SVG_PATH_COMMAND_UNKNOWN,
core/crates/io_svg/src/transform.rs:2:use craftcad_io::reasons::{AppError, ReasonCode};
core/crates/io_svg/src/transform.rs:140:                ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/transform.rs:154:                ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/io_svg/src/transform.rs:233:                    ReasonCode::IO_PARSE_SVG_MALFORMED,
core/crates/diagnostics/tests/basic.rs:1:use craftcad_diagnostics::*;
core/crates/diagnostics/tests/basic.rs:17:    let consent = DefaultDenyConsent;
core/crates/diagnostics/tests/basic.rs:18:    let limits = Limits::conservative_default();
core/crates/diagnostics/tests/basic.rs:34:        limits_profile: "default".into(),
core/crates/diagnostics/tests/basic.rs:37:    let mut b = JobLogBuilder::new(ctx, &red, &consent, limits);
core/crates/diagnostics/tests/basic.rs:50:    let mut b2 = JobLogBuilder::new(ctx2, &red, &consent, Limits::conservative_default());
core/crates/diagnostics/tests/basic.rs:75:    let limits = Limits::conservative_default();
core/crates/diagnostics/tests/basic.rs:76:    let mut o = OpLogBuilder::start_session("sess-1", &red, limits);
core/crates/diagnostics/tests/basic.rs:108:fn limits_truncate_inputs_and_params() {
core/crates/diagnostics/tests/basic.rs:110:    let consent = DefaultDenyConsent;
core/crates/diagnostics/tests/basic.rs:111:    let mut limits = Limits::conservative_default();
core/crates/diagnostics/tests/basic.rs:112:    limits.max_inputs = 1;
core/crates/diagnostics/tests/basic.rs:113:    limits.max_string_len = 4;
core/crates/diagnostics/tests/basic.rs:129:        limits_profile: "test".into(),
core/crates/diagnostics/tests/basic.rs:132:    let mut b = JobLogBuilder::new(ctx, &red, &consent, limits);
core/crates/diagnostics/Cargo.toml:2:name = "craftcad_diagnostics"
core/crates/diagnostics/src/support_zip.rs:13:    consent: Option<ConsentState>,
core/crates/diagnostics/src/support_zip.rs:28:            consent: None,
core/crates/diagnostics/src/support_zip.rs:47:    pub fn attach_consent(mut self, consent: ConsentState) -> Self {
core/crates/diagnostics/src/support_zip.rs:48:        self.consent = Some(consent);
core/crates/diagnostics/src/support_zip.rs:73:            .consent
core/crates/diagnostics/src/support_zip.rs:78:        if let Some(consent) = self.consent {
core/crates/diagnostics/src/support_zip.rs:79:            zip.start_file("consent.json", opts)
core/crates/diagnostics/src/support_zip.rs:81:            zip.write_all(&serde_json::to_vec_pretty(&consent).map_err(|e| e.to_string())?)
core/crates/security/src/lib.rs:1:pub mod consent;
core/crates/security/src/lib.rs:2:pub mod limits;
core/crates/security/src/lib.rs:3:pub mod redaction;
core/crates/security/src/lib.rs:6:pub use consent::ConsentState;
core/crates/security/src/lib.rs:7:pub use limits::{load_limits, SecurityLimits};
core/crates/security/src/lib.rs:8:pub use redaction::{redact_json, redact_str};
core/crates/security/src/limits.rs:13:pub fn load_limits(path: impl AsRef<Path>) -> Result<SecurityLimits, String> {
core/crates/diagnostics/src/joblog.rs:42:    pub limits_profile: String,
core/crates/diagnostics/src/joblog.rs:127:    pub consent_snapshot: ConsentSnapshot,
core/crates/diagnostics/src/joblog.rs:141:            limits_profile: self.header.limits_profile.clone(),
core/crates/diagnostics/src/joblog.rs:156:    pub limits_profile: String,
core/crates/diagnostics/src/joblog.rs:169:    consent: &'a dyn ConsentProvider,
core/crates/diagnostics/src/joblog.rs:170:    limits: Limits,
core/crates/diagnostics/src/joblog.rs:185:        consent: &'a dyn ConsentProvider,
core/crates/diagnostics/src/joblog.rs:186:        limits: Limits,
core/crates/diagnostics/src/joblog.rs:198:            limits_profile: ctx.limits_profile,
core/crates/diagnostics/src/joblog.rs:202:            consent,
core/crates/diagnostics/src/joblog.rs:203:            limits,
core/crates/diagnostics/src/joblog.rs:221:        if self.inputs.len() >= self.limits.max_inputs {
core/crates/diagnostics/src/joblog.rs:261:        if serialized.len() > self.limits.max_string_len * 4 {
core/crates/diagnostics/src/joblog.rs:355:        if self.steps.len() > self.limits.max_steps {
core/crates/diagnostics/src/joblog.rs:356:            self.steps.truncate(self.limits.max_steps);
core/crates/diagnostics/src/joblog.rs:383:            consent_snapshot: ConsentSnapshot {
core/crates/diagnostics/src/joblog.rs:384:                include_project_snapshot: self.consent.include_project_snapshot(),
core/crates/diagnostics/src/joblog.rs:385:                include_inputs_copy: self.consent.include_inputs_copy(),
core/crates/diagnostics/src/joblog.rs:386:                telemetry_opt_in: self.consent.telemetry_opt_in(),
core/crates/diagnostics/src/joblog.rs:406:        if self.reason_codes.len() >= self.builder.limits.max_reasons_per_step {
core/crates/diagnostics/src/joblog.rs:436:        if self.builder.steps.len() >= self.builder.limits.max_steps {
core/crates/diagnostics/src/lib.rs:12:pub use reason_summary::{EmptyCatalogLookup, ReasonCatalogLookup, ReasonSummary};
core/crates/diagnostics/src/reason_summary.rs:30:pub trait ReasonCatalogLookup {
core/crates/diagnostics/src/reason_summary.rs:35:impl ReasonCatalogLookup for EmptyCatalogLookup {
core/crates/diagnostics/src/reason_summary.rs:44:        catalog: &dyn ReasonCatalogLookup,
core/crates/diagnostics/src/oplog.rs:58:    limits: Limits,
core/crates/diagnostics/src/oplog.rs:66:    pub fn start_session(session_id: &str, redactor: &'a dyn Redactor, limits: Limits) -> Self {
core/crates/diagnostics/src/oplog.rs:70:            limits,
core/crates/diagnostics/src/oplog.rs:87:        if self.actions.len() >= self.limits.max_steps {
core/crates/diagnostics/src/oplog.rs:97:        if s.len() > self.limits.max_string_len * 4 {
core/crates/diagnostics/src/oplog.rs:104:        ids.truncate(self.limits.max_affected_ids);
core/crates/diagnostics/src/oplog.rs:111:        for rc in reason_codes.iter().take(self.limits.max_reasons_per_step) {
core/crates/diagnostics/src/repro.rs:48:    let _ = writeln!(&mut s, "- limits_profile: {}", joblog.header.limits_profile);
core/crates/diagnostics/src/repro.rs:97:        let _ = writeln!(&mut s, "- diagnostics_zip: {}", a.zip_name);
core/crates/diagnostics/src/repro.rs:98:        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: {}", a.zip_sha256);
core/crates/diagnostics/src/repro.rs:100:        let _ = writeln!(&mut s, "- diagnostics_zip: (not generated)");
core/crates/diagnostics/src/repro.rs:101:        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: (n/a)");
core/crates/perf/src/budgets.rs:2:use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
core/crates/perf/src/budgets.rs:82:            ReasonCode::new("PERF_BUDGET_LOAD_FAILED"),
core/crates/perf/src/budgets.rs:94:            ReasonCode::new("PERF_BUDGET_SCHEMA_INVALID"),
core/crates/perf/src/budgets.rs:133:                    ReasonCode::new("PERF_BUDGET_EXCEEDED_OPEN_P95"),
core/crates/perf/src/budgets.rs:146:                    ReasonCode::new("PERF_BUDGET_EXCEEDED_RENDER_P95"),
core/crates/perf/src/budgets.rs:160:                        ReasonCode::new("PERF_BUDGET_EXCEEDED_IO_IMPORT_P95"),
core/crates/perf/src/budgets.rs:175:                        ReasonCode::new("PERF_BUDGET_EXCEEDED_IO_EXPORT_P95"),
core/crates/perf/src/budgets.rs:189:                    ReasonCode::new("PERF_BUDGET_EXCEEDED_MAX_RSS"),
core/crates/perf/src/lib.rs:3:// - No panics on untrusted inputs. Always return ReasonCode + context.
core/crates/constraints/src/solver.rs:2:use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
core/crates/constraints/src/solver.rs:7:            ReasonCode::new("CAD_CONSTRAINT_POLICY_INVALID"),
core/crates/diycad_ffi/src/lib.rs:2:use diycad_common::{collect_basic_diagnostics, init_logging, log_info};
core/crates/diycad_ffi/src/lib.rs:19:        let diagnostics = collect_basic_diagnostics();
core/crates/diycad_ffi/src/lib.rs:20:        CString::new(diagnostics.app_version).expect("version string must not contain NUL")
core/crates/io/src/preflight.rs:2:use crate::reasons::{AppError, AppResult, ReasonCode};
core/crates/io/src/preflight.rs:6:    if len > opts.limits.max_bytes {
core/crates/io/src/preflight.rs:8:            ReasonCode::IO_LIMIT_BYTES_EXCEEDED,
core/crates/io/src/preflight.rs:11:                len, opts.limits.max_bytes
core/crates/io/src/preflight.rs:16:        .with_context("max_bytes", opts.limits.max_bytes.to_string())
core/crates/io/src/report.rs:21:    pub limits_triggered: Vec<String>,
core/crates/io/src/report.rs:43:            limits_triggered: vec![],
core/crates/io/src/lib.rs:17:use reasons::{AppError, AppResult, ReasonCode};
core/crates/io/src/lib.rs:81:            AppError::new(ReasonCode::IO_FORMAT_NOT_REGISTERED, "importer not found")
core/crates/io/src/lib.rs:105:            AppError::new(ReasonCode::IO_FORMAT_NOT_REGISTERED, "exporter not found")
core/crates/io/src/approx.rs:2:use crate::reasons::{AppError, ReasonCode};
core/crates/io/src/approx.rs:49:                                ReasonCode::IO_CURVE_APPROX_APPLIED,
core/crates/io/src/options.rs:66:    pub limits: Limits,
core/crates/io/src/options.rs:76:            limits: Limits::default_for_tests(),
core/crates/io/src/options.rs:133:            limits: Limits {
core/crates/io/src/normalize.rs:2:use crate::reasons::{AppError, ReasonCode};
core/crates/io/src/normalize.rs:106:                        ReasonCode::IO_SANITIZE_NONFINITE,
core/crates/io/src/normalize.rs:118:                        ReasonCode::IO_SANITIZE_NONFINITE,
core/crates/io/src/path_opt.rs:2:use crate::reasons::{AppError, ReasonCode};
core/crates/io/src/path_opt.rs:109:                        ReasonCode::IO_PATH_JOIN_APPLIED,
core/crates/io/src/reasons.rs:8:pub enum ReasonCode {
core/crates/io/src/reasons.rs:67:    pub reason: ReasonCode,
core/crates/io/src/reasons.rs:75:    pub fn new(reason: ReasonCode, message: impl Into<String>) -> Self {
core/crates/io/src/reasons.rs:104:    pub reason: ReasonCode,
core/crates/render_ir/src/reasons.rs:11:    /// Rendering degraded due to load/limits; quality reduced but deterministic.
core/crates/ssot_lint/tests/determinism_migrate.rs:37:        limits_ref: ds.limits_ref.clone(),
core/crates/ssot_lint/tests/golden_datasets.rs:48:        limits_ref: ds.limits_ref.clone(),
core/crates/ssot_lint/tests/golden_datasets.rs:164:                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
core/crates/ssot_lint/tests/golden_datasets.rs:170:                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
core/crates/ssot_lint/tests/determinism_io.rs:37:        limits_ref: ds.limits_ref.clone(),
core/crates/ssot_lint/tests/determinism_wizard.rs:37:        limits_ref: ds.limits_ref.clone(),
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:123:        "eviction should be recorded for diagnostics"
core/crates/ssot_lint/tests/golden_harness_smoke.rs:34:        limits_ref: "default".to_string(),
core/crates/ssot_lint/tests/golden_harness_smoke.rs:115:        limits_ref: "default".to_string(),
core/crates/cache/src/reasons.rs:7:/// Stable reason code for cache diagnostics.
core/crates/library/src/index.rs:1:use crate::reasons::{LibraryReason, LibraryReasonCode};
core/crates/library/src/index.rs:205:                LibraryReasonCode::LibIndexCorrupt,
core/crates/library/src/lib.rs:8:use crate::reasons::{LibraryReason, LibraryReasonCode};
core/crates/library/src/lib.rs:41:                LibraryReasonCode::LibIoError,
core/crates/library/src/lib.rs:64:            LibraryReason::new(LibraryReasonCode::LibIoError, format!("read failed: {e}"))
core/crates/library/src/lib.rs:69:                LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/deps.rs:1:use crate::reasons::{LibraryReason, LibraryReasonCode};
core/crates/library/src/deps.rs:39:                LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/deps.rs:51:                LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/deps.rs:63:                LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/deps.rs:75:                LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/deps.rs:89:            LibraryReasonCode::LibDepsMissingPreset,
core/crates/library/src/store.rs:2:use crate::reasons::{LibraryReason, LibraryReasonCode};
core/crates/library/src/store.rs:23:                LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:41:            LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:49:            LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:57:        LibraryReason::new(LibraryReasonCode::LibIoError, format!("write failed: {e}"))
core/crates/library/src/store.rs:61:        LibraryReason::new(LibraryReasonCode::LibIoError, format!("flush failed: {e}"))
core/crates/library/src/store.rs:68:                LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:77:            LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:100:                        LibraryReasonCode::LibIndexCorrupt,
core/crates/library/src/store.rs:108:                    LibraryReasonCode::LibIoError,
core/crates/library/src/store.rs:118:        LibraryReasonCode::LibIndexRebuilt,
core/crates/library/src/reasons.rs:4:pub enum LibraryReasonCode {
core/crates/library/src/reasons.rs:16:    pub code: LibraryReasonCode,
core/crates/library/src/reasons.rs:23:    pub fn new(code: LibraryReasonCode, message: impl Into<String>) -> Self {
core/crates/library/src/tags.rs:1:use crate::reasons::{LibraryReason, LibraryReasonCode};
core/crates/library/src/tags.rs:47:        LibraryReason::new(LibraryReasonCode::LibIoError, format!("read failed: {e}"))
core/crates/library/src/tags.rs:52:            LibraryReasonCode::LibTemplateInvalid,
core/crates/library/src/tags.rs:94:                LibraryReasonCode::LibTagNormalized,
core/crates/library/src/tags.rs:104:                LibraryReasonCode::LibTagNormalized,
core/crates/library/src/tags.rs:114:                LibraryReasonCode::LibTagNormalized,
core/crates/library/src/tags.rs:124:                LibraryReasonCode::LibTagNormalized,
core/crates/library/src/tags.rs:135:            LibraryReasonCode::LibTagInvalid,
core/crates/library/src/tags.rs:141:            LibraryReasonCode::LibTagInvalid,
core/crates/library/src/tags.rs:148:                LibraryReasonCode::LibTagInvalid,
core/crates/library/src/tags.rs:162:            LibraryReasonCode::LibTagInvalid,
core/crates/errors/tests/reason_catalog_lint.rs:155:            "ReasonCode catalog consistency check failed with {} errors:\n{}",
core/crates/io_json/tests/io_roundtrip_golden.rs:2:use craftcad_io::reasons::ReasonCode;
core/crates/io_json/tests/io_roundtrip_golden.rs:86:        .any(|w| w.is_fatal && w.reason == ReasonCode::IO_PARSE_JSON_MALFORMED));
core/crates/errors/src/lib.rs:4:pub struct ReasonCode(pub Cow<'static, str>);
core/crates/errors/src/lib.rs:6:impl ReasonCode {
core/crates/errors/src/lib.rs:39:    pub fn new(code: ReasonCode, severity: Severity, message: impl Into<String>) -> Self {
core/crates/errors/src/lib.rs:65:        code: ReasonCode,
core/crates/errors/src/lib.rs:74:        code: ReasonCode,
core/crates/io_json/src/import.rs:4:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_json/src/import.rs:10:        AppError::new(ReasonCode::IO_PARSE_JSON_MALFORMED, "malformed json")
core/crates/io_json/src/import.rs:38:        ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/import.rs:45:        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "point.x must be number")
core/crates/io_json/src/import.rs:48:        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "point.y must be number")
core/crates/io_json/src/import.rs:55:        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "segment must be object")
core/crates/io_json/src/import.rs:58:        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "segment.kind required")
core/crates/io_json/src/import.rs:63:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "line.a required")
core/crates/io_json/src/import.rs:66:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "line.b required")
core/crates/io_json/src/import.rs:72:                || AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.center required"),
core/crates/io_json/src/import.rs:75:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.radius required")
core/crates/io_json/src/import.rs:81:                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.start_rad required")
core/crates/io_json/src/import.rs:84:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.end_rad required")
core/crates/io_json/src/import.rs:87:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "arc.ccw required")
core/crates/io_json/src/import.rs:99:                || AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "circle.center required"),
core/crates/io_json/src/import.rs:102:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "circle.radius required")
core/crates/io_json/src/import.rs:108:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.a required")
core/crates/io_json/src/import.rs:111:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.c1 required")
core/crates/io_json/src/import.rs:114:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.c2 required")
core/crates/io_json/src/import.rs:117:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "bezier.b required")
core/crates/io_json/src/import.rs:122:            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown segment kind")
core/crates/io_json/src/import.rs:130:        AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "entity must be object")
core/crates/io_json/src/import.rs:135:        .ok_or_else(|| AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "entity.type required"))?;
core/crates/io_json/src/import.rs:148:                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "path.stroke required")
core/crates/io_json/src/import.rs:167:                        ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/import.rs:177:                        ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/import.rs:212:                    AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "text.pos required")
core/crates/io_json/src/import.rs:240:            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown entity.type")
core/crates/io_json/src/schema.rs:1:use craftcad_io::reasons::{AppError, AppResult, ReasonCode};
core/crates/io_json/src/schema.rs:28:            ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/schema.rs:36:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "missing required key")
core/crates/io_json/src/schema.rs:47:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/schema.rs:53:            ReasonCode::IO_JSON_SCHEMA_UNSUPPORTED_VERSION,
core/crates/io_json/src/schema.rs:64:        .ok_or_else(|| AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "units must be string"))?;
core/crates/io_json/src/schema.rs:67:            AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "units must be mm|inch")
core/crates/io_json/src/schema.rs:74:            ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/schema.rs:84:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/schema.rs:91:                ReasonCode::IO_JSON_SCHEMA_INVALID,
core/crates/io_json/src/schema.rs:104:                AppError::new(ReasonCode::IO_JSON_SCHEMA_INVALID, "unknown top-level key")
core/crates/sketch/src/validate.rs:3:use craftcad_errors::{AppError, AppResult, ReasonCode, Severity};
core/crates/sketch/src/validate.rs:21:pub fn validate(doc: &SketchDoc, limits: &ValidateLimits) -> AppResult<()> {
core/crates/sketch/src/validate.rs:22:    if doc.entities.len() > limits.max_entities {
core/crates/sketch/src/validate.rs:24:            ReasonCode::new("CAD_LIMIT_ENTITIES"),
core/crates/sketch/src/validate.rs:33:                    ReasonCode::new("CAD_INVALID_RADIUS"),
core/crates/sketch/src/validate.rs:40:                    ReasonCode::new("CAD_INVALID_RADIUS"),
core/crates/sketch/src/validate.rs:45:            Entity::Polyline(p) if p.pts.len() > limits.max_polyline_points => {
core/crates/sketch/src/validate.rs:47:                    ReasonCode::new("CAD_LIMIT_POLYLINE_POINTS"),
core/crates/sketch/src/validate.rs:52:            Entity::Text(t) if t.text.len() > limits.max_text_len => {
core/crates/sketch/src/validate.rs:54:                    ReasonCode::new("CAD_LIMIT_TEXT_LEN"),
core/crates/diycad_common/src/diagnostics.rs:14:pub fn collect_basic_diagnostics() -> BasicDiagnostics {
core/crates/diycad_common/src/diagnostics.rs:34:    fn diagnostics_contains_core_fields() {
core/crates/diycad_common/src/diagnostics.rs:35:        let diagnostics = collect_basic_diagnostics();
core/crates/diycad_common/src/diagnostics.rs:36:        assert!(!diagnostics.os.is_empty());
core/crates/diycad_common/src/diagnostics.rs:37:        assert!(!diagnostics.arch.is_empty());
core/crates/diycad_common/src/diagnostics.rs:38:        assert!(!diagnostics.app_version.is_empty());
core/crates/diycad_common/src/diagnostics.rs:39:        assert!(!diagnostics.time.is_empty());
core/crates/diycad_common/src/lib.rs:2:pub mod diagnostics;
core/crates/diycad_common/src/lib.rs:10:pub use diagnostics::{collect_basic_diagnostics, BasicDiagnostics};
core/crates/diycad_format/tests/limits_zipbomb.rs:31:    opt.limits = Limits {
core/crates/diycad_format/tests/limits_zipbomb.rs:37:        .expect_err("expected limits error")
core/crates/diycad_format/src/package.rs:1:use crate::{validate_entry_path, LimitViolation, Limits, ReasonCode};
core/crates/diycad_format/src/package.rs:22:    pub fn open(mut zip: ZipArchive<R>, limits: &Limits) -> Result<Self> {
core/crates/diycad_format/src/package.rs:27:        if n > limits.max_entries {
core/crates/diycad_format/src/package.rs:30:                ReasonCode::SecZipTooManyEntries.as_str(),
core/crates/diycad_format/src/package.rs:32:                limits.max_entries
core/crates/diycad_format/src/package.rs:38:            let norm = match validate_entry_path(limits, f.name()) {
core/crates/diycad_format/src/package.rs:48:            if size > limits.max_entry_uncompressed {
core/crates/diycad_format/src/package.rs:51:                    ReasonCode::SecZipEntryTooLarge.as_str(),
core/crates/diycad_format/src/package.rs:54:                    limits.max_entry_uncompressed
core/crates/diycad_format/src/package.rs:58:            if total > limits.max_total_uncompressed {
core/crates/diycad_format/src/package.rs:61:                    ReasonCode::SecZipTotalUncompressedTooLarge.as_str(),
core/crates/diycad_format/src/package.rs:63:                    limits.max_total_uncompressed
core/crates/diycad_format/src/package.rs:92:                ReasonCode::SecZipEntryTooLarge.as_str(),
core/crates/diycad_format/src/package.rs:105:pub fn open_package_file(path: &Path, limits: &Limits) -> Result<PackageReader<File>> {
core/crates/diycad_format/src/package.rs:108:        ZipArchive::new(f).map_err(|e| anyhow!("{}: {}", ReasonCode::SecZipBadZip.as_str(), e))?;
core/crates/diycad_format/src/package.rs:109:    PackageReader::open(zip, limits)
core/crates/diycad_format/src/integrity.rs:2:    AppWarning, ContentEntry, ContentManifest, Manifest, ReasonCode, SalvageActionHint, WarningKind,
core/crates/diycad_format/src/integrity.rs:37:                code: ReasonCode::SaveIntegrityManifestMissing,
core/crates/diycad_format/src/integrity.rs:55:                    code: ReasonCode::SaveIntegrityEntryMissing,
core/crates/diycad_format/src/integrity.rs:66:                code: ReasonCode::SaveIntegritySizeMismatch,
core/crates/diycad_format/src/integrity.rs:76:                code: ReasonCode::SaveIntegrityShaMismatch,
core/crates/diycad_format/src/lib.rs:2:mod limits;
core/crates/diycad_format/src/lib.rs:11:pub use limits::*;
core/crates/diycad_nesting/src/pack.rs:3:    NestJob, PartPlacementStatus, PartPlacementStatusKind, Reason, ReasonCode,
core/crates/diycad_nesting/src/pack.rs:66:            ReasonCode::NestPartTooLargeForAnySheet
core/crates/diycad_nesting/src/pack.rs:68:            ReasonCode::NestNoFeasiblePositionWithMarginAndKerf
core/crates/diycad_nesting/src/constraints.rs:1:use craftcad_serialize::{Document, NestJob, Reason, ReasonCode, Result};
core/crates/diycad_nesting/src/constraints.rs:5:        return Err(Reason::from_code(ReasonCode::NestInternalInfeasible));
core/crates/diycad_nesting/src/constraints.rs:9:            return Err(Reason::from_code(ReasonCode::NestInternalInfeasible));
core/crates/diycad_nesting/src/constraints.rs:14:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/crates/diycad_nesting/src/lib.rs:11:    Document, NestJob, NestResultV1, NestTraceV1, PartPlacementStatus, Reason, ReasonCode, Result,
core/crates/diycad_nesting/src/lib.rs:38:    limits: RunLimits,
core/crates/diycad_nesting/src/lib.rs:49:    let iterations = limits.iteration_limit.max(1);
core/crates/diycad_nesting/src/lib.rs:52:        if start.elapsed().as_millis() as u64 >= limits.time_limit_ms {
core/crates/diycad_nesting/src/lib.rs:84:        ReasonCode::NestStoppedByTimeLimit.as_str().to_string()
core/crates/diycad_nesting/src/lib.rs:86:        ReasonCode::NestStoppedByIterationLimit.as_str().to_string()
core/crates/diycad_nesting/src/lib.rs:89:    let result = best.ok_or_else(|| Reason::from_code(ReasonCode::NestInternalInfeasible))?;
core/crates/diycad_format/src/open.rs:3:    OpenResult, Part, ReasonCode, SalvageActionHint, WarningKind, ZipIndexEntry,
core/crates/diycad_format/src/open.rs:17:    code: ReasonCode,
core/crates/diycad_format/src/open.rs:55:                ReasonCode::OpenDocumentLocateHeuristicUsed,
core/crates/diycad_format/src/open.rs:77:    let mut pkg = open_package_file(path, &opt.limits)?;
core/crates/diycad_format/src/open.rs:85:        .read_entry_bytes("manifest.json", opt.limits.max_entry_uncompressed)?
core/crates/diycad_format/src/open.rs:90:                ReasonCode::OpenManifestMissing,
core/crates/diycad_format/src/open.rs:104:                            ReasonCode::OpenSchemaForwardIncompatibleReadonly,
core/crates/diycad_format/src/open.rs:115:                            ReasonCode::OpenSchemaForwardIncompatibleReadonly.as_str(),
core/crates/diycad_format/src/open.rs:123:                        ReasonCode::OpenSchemaTooOldSuggestMigrate,
core/crates/diycad_format/src/open.rs:139:                    ReasonCode::OpenManifestInvalidJson,
core/crates/diycad_format/src/open.rs:153:                ReasonCode::OpenDocumentMissing.as_str()
core/crates/diycad_format/src/open.rs:157:        .read_entry_bytes(&doc_path, opt.limits.max_entry_uncompressed)?
core/crates/diycad_format/src/open.rs:161:                ReasonCode::OpenDocumentMissing.as_str(),
core/crates/diycad_format/src/open.rs:166:        .map_err(|e| anyhow!("{}: {}", ReasonCode::OpenDocumentInvalidJson.as_str(), e))?;
core/crates/diycad_format/src/open.rs:189:        .take(opt.limits.max_parts)
core/crates/diycad_format/src/open.rs:191:    if parts_total > opt.limits.max_parts {
core/crates/diycad_format/src/open.rs:195:            ReasonCode::SecZipTooManyEntries,
core/crates/diycad_format/src/open.rs:200:                opt.limits.max_parts
core/crates/diycad_format/src/open.rs:206:        let b = match pkg.read_entry_bytes(&p, opt.limits.max_entry_uncompressed) {
core/crates/diycad_format/src/open.rs:213:                    code: ReasonCode::IoReadFailed,
core/crates/diycad_format/src/open.rs:225:                    code: ReasonCode::OpenPartInvalidJson,
core/crates/diycad_format/src/open.rs:242:        .take(opt.limits.max_nest_jobs)
core/crates/diycad_format/src/open.rs:244:    if nest_jobs_total > opt.limits.max_nest_jobs {
core/crates/diycad_format/src/open.rs:248:            ReasonCode::SecZipTooManyEntries,
core/crates/diycad_format/src/open.rs:253:                opt.limits.max_nest_jobs
core/crates/diycad_format/src/open.rs:259:        let b = match pkg.read_entry_bytes(&p, opt.limits.max_entry_uncompressed) {
core/crates/diycad_format/src/open.rs:266:                    code: ReasonCode::IoReadFailed,
core/crates/diycad_format/src/open.rs:278:                    code: ReasonCode::OpenNestJobInvalidJson,
core/crates/diycad_format/src/open.rs:334:                        ReasonCode::MigrateApplied,
core/crates/diycad_format/src/open.rs:345:                        ReasonCode::MigrateFailed,
core/crates/diycad_format/src/open.rs:358:                pkg.read_entry_bytes(p, opt.limits.max_entry_uncompressed)
core/crates/diycad_format/src/open.rs:367:                ReasonCode::SaveIntegrityManifestMissing,
core/crates/diycad_format/src/reasons.rs:2:pub enum ReasonCode {
core/crates/diycad_format/src/reasons.rs:3:    // Security / limits
core/crates/diycad_format/src/reasons.rs:55:impl ReasonCode {
core/crates/diycad_format/src/reasons.rs:57:        use ReasonCode::*;
core/crates/diycad_nesting/src/model.rs:2:    BBox, Document, NestJob, Part, PartRef, Placement, Reason, ReasonCode, Result,
core/crates/diycad_nesting/src/model.rs:60:        return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
core/crates/diycad_nesting/src/model.rs:68:            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
core/crates/diycad_nesting/src/model.rs:92:                let mut reason = Reason::from_code(ReasonCode::ModelReferenceNotFound);
core/crates/diycad_format/src/limits.rs:1:use crate::ReasonCode;
core/crates/diycad_format/src/limits.rs:30:    pub code: ReasonCode,
core/crates/diycad_format/src/limits.rs:45:            code: ReasonCode::SecZipInvalidEntryName,
core/crates/diycad_format/src/limits.rs:51:            code: ReasonCode::SecZipInvalidEntryName,
core/crates/diycad_format/src/limits.rs:57:            code: ReasonCode::SecZipAbsolutePath,
core/crates/diycad_format/src/limits.rs:63:            code: ReasonCode::SecZipPathTooLong,
core/crates/diycad_format/src/limits.rs:70:            code: ReasonCode::SecZipPathTooDeep,
core/crates/diycad_format/src/limits.rs:77:                code: ReasonCode::SecZipTraversal,
core/crates/diycad_format/src/types.rs:1:use crate::ReasonCode;
core/crates/diycad_format/src/types.rs:106:    pub code: ReasonCode,
core/crates/diycad_format/src/types.rs:115:    pub code: ReasonCode,
core/crates/diycad_format/src/types.rs:148:    pub limits: crate::Limits,
core/crates/diycad_format/src/types.rs:158:            limits: crate::Limits::default(),
core/crates/diycad_format/src/save.rs:2:    build_content_manifest, ContentManifest, Document, Manifest, NestJob, Part, ReasonCode,
core/crates/diycad_format/src/save.rs:62:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicFsyncFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:65:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicRenameFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:82:                ReasonCode::SaveValidateFailed.as_str()
core/crates/diycad_format/src/save.rs:88:                ReasonCode::SaveValidateFailed.as_str()
core/crates/diycad_format/src/save.rs:102:                ReasonCode::SaveValidateFailed.as_str()
core/crates/diycad_format/src/save.rs:179:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicTempCreateFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:184:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:186:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:189:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:191:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:195:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:197:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:201:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:203:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:207:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:209:            .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:214:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicWriteFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:217:        .map_err(|e| anyhow!("{}: {}", ReasonCode::SaveAtomicFsyncFailed.as_str(), e))?;
core/crates/diycad_format/src/save.rs:223:            .map_err(|e| anyhow!("{}: {}", ReasonCode::IoWriteFailed.as_str(), e))?;
core/commands/tests/e2e_pipeline.rs:7:    Part, PartRef, Polygon2D, ProjectSettings, ReasonCode, SheetDef, Vec2,
core/commands/tests/e2e_pipeline.rs:162:    assert_eq!(err.code, ReasonCode::ModelReferenceNotFound.as_str());
core/commands/tests/nesting_roundtrip.rs:96:        limits: RunLimits {
core/commands/src/commands/offset_entity.rs:2:use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result};
core/commands/src/commands/offset_entity.rs:42:            return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
core/commands/src/commands/offset_entity.rs:51:            .ok_or_else(|| Reason::from_code(ReasonCode::EditInvalidNumeric))?;
core/commands/src/commands/offset_entity.rs:78:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/offset_entity.rs:80:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/offset_entity.rs:86:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/offset_entity.rs:88:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/offset_entity.rs:96:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/offset_entity.rs:102:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/offset_entity.rs:107:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/offset_entity.rs:109:                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
core/commands/src/commands/offset_entity.rs:123:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/offset_entity.rs:125:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/offset_entity.rs:135:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/create_shapes.rs:2:use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/create_shapes.rs:8:        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/commands/src/commands/create_shapes.rs:17:        Err(Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/commands/src/commands/create_shapes.rs:69:                    return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/commands/src/commands/create_shapes.rs:80:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/create_shapes.rs:143:                    return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/commands/src/commands/create_shapes.rs:154:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/create_shapes.rs:216:                    return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/commands/src/commands/create_shapes.rs:227:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/create_shapes.rs:285:            return Err(Reason::from_code(ReasonCode::DrawInsufficientInput));
core/commands/src/commands/create_shapes.rs:288:            return Err(Reason::from_code(ReasonCode::DrawInsufficientInput));
core/commands/src/commands/create_shapes.rs:300:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/create_shapes.rs:332:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/transform_selection.rs:3:use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/transform_selection.rs:44:            return Err(Reason::from_code(ReasonCode::EditNoSelection));
core/commands/src/commands/transform_selection.rs:53:            .ok_or_else(|| Reason::from_code(ReasonCode::EditNoSelection))?;
core/commands/src/commands/transform_selection.rs:99:            return Err(Reason::from_code(ReasonCode::EditNoSelection));
core/commands/src/commands/transform_selection.rs:107:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/transform_selection.rs:112:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/transform_selection.rs:114:                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
core/commands/src/commands/transform_selection.rs:131:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/transform_selection.rs:133:                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
core/commands/src/commands/transform_selection.rs:139:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/transform_selection.rs:141:                return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
core/commands/src/commands/transform_selection.rs:158:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/transform_selection.rs:164:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/transform_selection.rs:171:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/transform_selection.rs:174:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/create_line.rs:2:use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/create_line.rs:29:            return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
core/commands/src/commands/create_line.rs:54:            .ok_or_else(|| Reason::from_code(ReasonCode::GeomInvalidNumeric))?;
core/commands/src/commands/create_line.rs:85:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/create_line.rs:95:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/advanced_edit.rs:2:use craftcad_serialize::{Document, Entity, Geom2D, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/advanced_edit.rs:11:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/advanced_edit.rs:13:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/advanced_edit.rs:18:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/advanced_edit.rs:20:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/advanced_edit.rs:34:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/advanced_edit.rs:38:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/advanced_edit.rs:47:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/advanced_edit.rs:54:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/advanced_edit.rs:57:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/advanced_edit.rs:91:                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:96:                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:102:                    _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/commands/src/commands/advanced_edit.rs:179:                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:184:                    .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:190:                    _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/commands/src/commands/advanced_edit.rs:199:                .map_err(|_| Reason::from_code(ReasonCode::EditChamferDistanceTooLarge))?;
core/commands/src/commands/advanced_edit.rs:228:                        .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:241:                    .map_err(|_| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))?;
core/commands/src/commands/advanced_edit.rs:257:                        .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/advanced_edit.rs:328:            return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/commands/src/commands/advanced_edit.rs:337:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/advanced_edit.rs:381:            return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/commands/src/commands/advanced_edit.rs:390:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/advanced_edit.rs:440:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/advanced_edit.rs:500:            _ => return Err(Reason::from_code(ReasonCode::EditPatternInvalidParams)),
core/commands/src/commands/advanced_edit.rs:509:            .ok_or_else(|| Reason::from_code(ReasonCode::DrawInsufficientInput))?;
core/commands/src/commands/create_part.rs:4:use craftcad_serialize::{Document, Part, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/create_part.rs:59:            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
core/commands/src/commands/create_part.rs:67:            return Err(Reason::from_code(ReasonCode::PartInvalidFields));
core/commands/src/commands/create_part.rs:93:            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidOutline))?;
core/commands/src/commands/create_part.rs:130:            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidOutline))?;
core/commands/src/commands/create_part.rs:182:            return Err(Reason::from_code(ReasonCode::PartInvalidFields));
core/commands/src/commands/create_part.rs:192:            .ok_or_else(|| Reason::from_code(ReasonCode::PartInvalidFields))?;
core/commands/src/commands/create_part.rs:231:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/create_part.rs:253:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/create_part.rs:270:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/create_part.rs:279:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/create_part.rs:294:            return Err(Reason::from_code(ReasonCode::ModelReferenceNotFound));
core/commands/src/commands/trim_entity.rs:2:use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
core/commands/src/commands/trim_entity.rs:41:            return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
core/commands/src/commands/trim_entity.rs:50:            .ok_or_else(|| Reason::from_code(ReasonCode::EditInvalidNumeric))?;
core/commands/src/commands/trim_entity.rs:79:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/trim_entity.rs:81:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/trim_entity.rs:86:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?,
core/commands/src/commands/trim_entity.rs:88:    .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))
core/commands/src/commands/trim_entity.rs:96:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/trim_entity.rs:102:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/trim_entity.rs:107:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/trim_entity.rs:112:                .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/trim_entity.rs:114:                return Err(Reason::from_code(ReasonCode::EditTargetLockedOrHidden));
core/commands/src/commands/trim_entity.rs:137:                _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/commands/src/commands/trim_entity.rs:143:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/trim_entity.rs:148:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/trim_entity.rs:150:            return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
core/commands/src/commands/trim_entity.rs:160:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/trim_entity.rs:163:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/trim_entity.rs:168:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/trim_entity.rs:170:            return Err(Reason::from_code(ReasonCode::CoreInvariantViolation));
core/commands/src/commands/nesting.rs:2:use craftcad_serialize::{Document, NestResultV1, NestTraceV1, Reason, ReasonCode, Result};
core/commands/src/commands/nesting.rs:11:    pub limits: RunLimits,
core/commands/src/commands/nesting.rs:40:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/commands/src/commands/nesting.rs:43:            run_nesting(&before_job, &input.doc_snapshot, &input.eps, input.limits)?;
core/commands/src/commands/nesting.rs:55:            Reason::from_code(ReasonCode::CoreInvariantViolation)
core/commands/src/commands/nesting.rs:78:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:88:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:142:            Reason::from_code(ReasonCode::CoreInvariantViolation)
core/commands/src/commands/nesting.rs:165:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:169:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:176:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:192:            .ok_or_else(|| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/commands/src/commands/nesting.rs:197:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:201:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/commands/src/commands/nesting.rs:208:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/crates/diycad_geom/src/ops/intersect.rs:2:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/intersect.rs:81:            let mut reason = Reason::from_code(ReasonCode::GeomIntersectionAmbiguous);
core/crates/diycad_geom/src/ops/intersect.rs:87:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:96:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:106:        return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
core/crates/diycad_geom/src/ops/intersect.rs:112:        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/intersect.rs:122:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:140:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:147:        return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
core/crates/diycad_geom/src/ops/intersect.rs:155:        let mut reason = Reason::from_code(ReasonCode::GeomIntersectionAmbiguous);
core/crates/diycad_geom/src/ops/intersect.rs:163:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:170:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:193:        return Err(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:207:        return Err(Reason::from_code(ReasonCode::GeomArcRangeInvalid));
core/crates/diycad_geom/src/ops/intersect.rs:285:            let mut reason = Reason::from_code(ReasonCode::GeomNoIntersection);
core/crates/diycad_geom/src/ops/intersect.rs:307:                    last_error = Some(Reason::from_code(ReasonCode::GeomNoIntersection));
core/crates/diycad_geom/src/ops/intersect.rs:327:                if reason.code == ReasonCode::GeomNoIntersection.as_str() {
core/crates/diycad_geom/src/ops/intersect.rs:343:        Err(last_error.unwrap_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection)))
core/crates/diycad_geom/src/ops/intersect.rs:345:        if last.code == ReasonCode::GeomNoIntersection.as_str() {
core/crates/diycad_geom/src/ops/intersect.rs:348:            Err(Reason::from_code(ReasonCode::GeomFallbackLimitReached))
core/crates/diycad_geom/src/ops/intersect.rs:351:        Err(Reason::from_code(ReasonCode::GeomFallbackLimitReached))
core/crates/diycad_geom/src/ops/edit.rs:2:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/edit.rs:42:    let u = norm(dir).ok_or_else(|| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))?;
core/crates/diycad_geom/src/ops/edit.rs:86:        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/crates/diycad_geom/src/ops/edit.rs:89:        .ok_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection))?;
core/crates/diycad_geom/src/ops/edit.rs:90:    let ua = norm(sub(a, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
core/crates/diycad_geom/src/ops/edit.rs:91:    let ub = norm(sub(c, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
core/crates/diycad_geom/src/ops/edit.rs:95:        return Err(Reason::from_code(ReasonCode::EditChamferDistanceTooLarge));
core/crates/diycad_geom/src/ops/edit.rs:112:        return Err(Reason::from_code(ReasonCode::DrawInvalidNumeric));
core/crates/diycad_geom/src/ops/edit.rs:115:        .ok_or_else(|| Reason::from_code(ReasonCode::GeomNoIntersection))?;
core/crates/diycad_geom/src/ops/edit.rs:116:    let ua = norm(sub(a, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
core/crates/diycad_geom/src/ops/edit.rs:117:    let ub = norm(sub(c, p)).ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))?;
core/crates/diycad_geom/src/ops/edit.rs:121:        return Err(Reason::from_code(ReasonCode::EditFilletRadiusTooLarge));
core/crates/diycad_geom/src/ops/edit.rs:127:        return Err(Reason::from_code(ReasonCode::EditFilletRadiusTooLarge));
core/crates/diycad_geom/src/ops/edit.rs:130:        norm(add(ua, ub)).ok_or_else(|| Reason::from_code(ReasonCode::EditFilletRadiusTooLarge))?;
core/edit_ops/src/lib.rs:3:use craftcad_serialize::{Geom2D, Reason, ReasonCode, Result, Vec2};
core/edit_ops/src/lib.rs:42:        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
core/edit_ops/src/lib.rs:75:        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
core/edit_ops/src/lib.rs:111:        return Err(Reason::from_code(ReasonCode::EditInvalidNumeric));
core/edit_ops/src/lib.rs:114:        return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
core/edit_ops/src/lib.rs:123:                return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
core/edit_ops/src/lib.rs:138:                return Err(Reason::from_code(ReasonCode::EditTransformWouldDegenerate));
core/crates/diycad_geom/src/ops/split.rs:4:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/split.rs:20:                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/split.rs:27:                        return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:33:                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:50:                return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
core/crates/diycad_geom/src/ops/split.rs:53:                return Err(Reason::from_code(ReasonCode::GeomArcRangeInvalid));
core/crates/diycad_geom/src/ops/split.rs:60:                        return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:66:                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:118:                let mut r = Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom);
core/crates/diycad_geom/src/ops/split.rs:127:                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/split.rs:134:                        return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:140:                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:146:                return Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom));
core/crates/diycad_geom/src/ops/split.rs:155:                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/split.rs:170:        _ => Err(Reason::from_code(ReasonCode::GeomSplitPointNotOnGeom)),
core/crates/diycad_geom/src/ops/trim.rs:2:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/trim.rs:23:        return Err(Reason::from_code(ReasonCode::GeomTrimNoIntersection));
core/crates/diycad_geom/src/ops/trim.rs:38:            let mut err = Reason::from_code(ReasonCode::EditTrimAmbiguousCandidate);
core/crates/diycad_geom/src/ops/trim.rs:54:        let mut err = Reason::from_code(ReasonCode::EditTrimAmbiguousCandidate);
core/crates/diycad_geom/src/ops/trim.rs:77:        _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/crates/diycad_geom/src/ops/trim.rs:81:            Reason::from_code(ReasonCode::GeomTrimNoIntersection)
core/crates/diycad_geom/src/ops/trim.rs:105:        _ => return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/crates/diycad_geom/src/ops/trim.rs:108:        return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
core/crates/diycad_geom/src/ops/project.rs:5:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/project.rs:78:        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/project.rs:97:                return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
core/crates/diycad_geom/src/ops/project.rs:126:                return Err(Reason::from_code(ReasonCode::GeomCircleRadiusInvalid));
core/crates/diycad_geom/src/ops/project.rs:129:                return Err(Reason::from_code(ReasonCode::GeomArcRangeInvalid));
core/crates/diycad_geom/src/ops/project.rs:183:                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/project.rs:204:            best.ok_or_else(|| Reason::from_code(ReasonCode::GeomDegenerate))
core/crates/diycad_geom/src/ops/offset.rs:5:use craftcad_serialize::{Reason, ReasonCode, Result};
core/crates/diycad_geom/src/ops/offset.rs:11:        return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/offset.rs:31:        return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
core/crates/diycad_geom/src/ops/offset.rs:40:        return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
core/crates/diycad_geom/src/ops/offset.rs:53:                return Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported));
core/crates/diycad_geom/src/ops/offset.rs:56:                return Err(Reason::from_code(ReasonCode::GeomDegenerate));
core/crates/diycad_geom/src/ops/offset.rs:80:                    return Err(Reason::from_code(ReasonCode::GeomOffsetSelfIntersection));
core/crates/diycad_geom/src/ops/offset.rs:89:        _ => Err(Reason::from_code(ReasonCode::GeomOffsetNotSupported)),
core/serialize/tests/spec_ssot_lint.rs:514:            "{REASON_CATALOG_JSON}: unknown ReasonCatalog structure (expected 'items' array, 'reasons' array or 'codes' object)"
core/serialize/tests/spec_ssot_lint.rs:564:                        "{SUPPORT_MATRIX_JSON}: unknown ReasonCode '{code}' (not found in {REASON_CATALOG_JSON})"
core/serialize/tests/spec_ssot_lint.rs:1136:fn ssot_diagnostics_contracts_exist_and_valid() {
core/serialize/tests/spec_ssot_lint.rs:1138:    let dir = root.join("docs").join("specs").join("diagnostics");
core/serialize/tests/spec_ssot_lint.rs:1141:        "missing diagnostics ssot dir: {}",
core/serialize/tests/spec_ssot_lint.rs:1158:            "missing required diagnostics spec file: {}",
core/crates/presets/src/salvage.rs:2:use crate::reasons::{PresetReason, PresetReasonCode};
core/crates/presets/src/salvage.rs:23:                PresetReasonCode::PresetSemverInvalid,
core/crates/presets/src/reasons.rs:4:pub enum PresetReasonCode {
core/crates/presets/src/reasons.rs:10:    pub code: PresetReasonCode,
core/crates/presets/src/reasons.rs:15:    pub fn new(code: PresetReasonCode, message: impl Into<String>) -> Self {
core/bom/src/lib.rs:3:use craftcad_serialize::{Document, Reason, ReasonCode, Result};
core/bom/src/lib.rs:78:            let mut r = Reason::from_code(ReasonCode::MaterialNotFound);
core/serialize/src/lib.rs:28:    pub fn from_code(code: ReasonCode) -> Self {
core/serialize/src/lib.rs:38:pub enum ReasonCode {
core/serialize/src/lib.rs:91:impl ReasonCode {
core/serialize/src/lib.rs:472:        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
core/serialize/src/lib.rs:476:        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
core/serialize/src/lib.rs:485:            Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
core/serialize/src/lib.rs:494:        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
core/serialize/src/lib.rs:556:        Reason::from_code(ReasonCode::SerializeSchemaValidationFailed)
core/serialize/src/lib.rs:583:        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("io", e.to_string())
core/serialize/src/lib.rs:589:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:593:        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("zip", e.to_string())
core/serialize/src/lib.rs:596:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:602:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:606:        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("zip", e.to_string())
core/serialize/src/lib.rs:609:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:616:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:624:        Reason::from_code(ReasonCode::SerializePackageCorrupted).with_debug("io", e.to_string())
core/serialize/src/lib.rs:627:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:634:            Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:638:            Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:645:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:651:            Reason::from_code(ReasonCode::SerializeUnsupportedSchemaVersion)
core/serialize/src/lib.rs:659:            Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:663:            Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/serialize/src/lib.rs:670:        Reason::from_code(ReasonCode::SerializePackageCorrupted)
core/ffi_desktop/include/craftcad_desktop_ffi.h:47:char *craftcad_history_apply_run_nesting(uint64_t h, const char *doc_json, const char *job_id_uuid, const char *eps_json, const char *limits_json);
core/ffi_desktop/src/lib.rs:40:use craftcad_serialize::{load_diycad, Document, Part, Reason, ReasonCode, Vec2};
core/ffi_desktop/src/lib.rs:140:        let mut r = Reason::from_code(ReasonCode::SerializePackageCorrupted);
core/ffi_desktop/src/lib.rs:148:        .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:180:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:186:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:217:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidOutline))
core/ffi_desktop/src/lib.rs:241:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidOutline))
core/ffi_desktop/src/lib.rs:247:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))
core/ffi_desktop/src/lib.rs:275:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:281:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))
core/ffi_desktop/src/lib.rs:292:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/ffi_desktop/src/lib.rs:295:            .map_err(|_| Reason::from_code(ReasonCode::CoreInvariantViolation))?;
core/ffi_desktop/src/lib.rs:298:            .map_err(|_| Reason::from_code(ReasonCode::PartInvalidFields))?;
core/ffi_desktop/src/lib.rs:316:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:327:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?
core/ffi_desktop/src/lib.rs:345:    limits_json: *const c_char,
core/ffi_desktop/src/lib.rs:348:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:354:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:359:    let limits: RunLimits = match parse_cstr(limits_json, "limits_json").and_then(|s| {
core/ffi_desktop/src/lib.rs:361:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:373:            limits,
core/ffi_desktop/src/lib.rs:393:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:399:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:405:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
core/ffi_desktop/src/lib.rs:421:            .ok_or_else(|| Reason::from_code(ReasonCode::ModelReferenceNotFound))?;
core/ffi_desktop/src/lib.rs:450:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:456:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:478:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:484:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:506:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:512:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::ExportIoParseFailed))
core/ffi_desktop/src/lib.rs:535:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:541:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::BomExportFailed))
core/ffi_desktop/src/lib.rs:582:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:584:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:586:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:589:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:604:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:606:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:608:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:611:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:626:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:628:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))?;
core/ffi_desktop/src/lib.rs:631:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:688:            .map_err(|_| Reason::from_code(ReasonCode::SerializePackageCorrupted))
core/ffi_desktop/src/lib.rs:695:        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
core/ffi_desktop/src/lib.rs:699:        None => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
core/ffi_desktop/src/lib.rs:721:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:728:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:732:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:762:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:769:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:773:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:803:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditMirrorAxisInvalid))
core/ffi_desktop/src/lib.rs:812:            Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:844:            .map_err(|_| Reason::from_code(ReasonCode::EditPatternInvalidParams))
core/ffi_desktop/src/lib.rs:853:            Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:875:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditAmbiguousCandidate))
core/ffi_desktop/src/lib.rs:896:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:902:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:927:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:934:            serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:959:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:965:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:990:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:997:            serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::DrawInvalidNumeric))
core/ffi_desktop/src/lib.rs:1022:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:1028:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:1034:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:1064:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
core/ffi_desktop/src/lib.rs:1075:                Err(_) => return encode_err(Reason::from_code(ReasonCode::ModelReferenceNotFound)),
core/ffi_desktop/src/lib.rs:1083:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::EditInvalidNumeric))
core/ffi_desktop/src/lib.rs:1112:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:1118:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:1170:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:1176:        Uuid::parse_str(&s).map_err(|_| Reason::from_code(ReasonCode::ModelReferenceNotFound))
core/ffi_desktop/src/lib.rs:1182:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:1188:        serde_json::from_str(&s).map_err(|_| Reason::from_code(ReasonCode::GeomInvalidNumeric))
core/ffi_desktop/src/lib.rs:1235:        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
core/ffi_desktop/src/lib.rs:1241:    encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation))
core/ffi_desktop/src/lib.rs:1248:        Err(_) => return encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation)),
core/ffi_desktop/src/lib.rs:1254:    encode_err(Reason::from_code(ReasonCode::CoreInvariantViolation))
core/ffi_desktop/src/lib.rs:1273:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ExportIoParseFailed)),
core/ffi_desktop/src/lib.rs:1314:        nesting_limits: opts_v.get("nesting_limits").cloned(),
core/ffi_desktop/src/lib.rs:1320:        Err(_) => return encode_err(Reason::from_code(ReasonCode::ExportIoWriteFailed)),
core/Cargo.toml:29:  "crates/diagnostics",
core/export/src/svg.rs:1:use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
core/export/src/svg.rs:61:                _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
core/export/src/pdf_tiled.rs:1:use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result, Vec2};
core/export/src/pdf_tiled.rs:68:        _ => Err(Reason::from_code(ReasonCode::ExportUnsupportedFeature)),
core/export/src/pdf_tiled.rs:87:        return Err(Reason::from_code(ReasonCode::ExportUnsupportedFeature));
core/export/src/pdf_tiled.rs:95:            return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity));
core/export/src/pdf_tiled.rs:146:            _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
core/export/src/pdf_drawing.rs:1:use craftcad_serialize::{Document, Geom2D, Reason, ReasonCode, Result};
core/export/src/pdf_drawing.rs:20:            _ => return Err(Reason::from_code(ReasonCode::ExportUnsupportedEntity)),
core/diag/src/lib.rs:20:    pub nesting_limits: Option<Value>,
core/diag/src/lib.rs:35:            nesting_limits: None,
core/diag/src/lib.rs:82:            "nesting_limits": options.nesting_limits,
core/tests/determinism_migrate.rs:26:        limits_ref: ds.limits_ref.clone(),
core/tests/golden_datasets.rs:34:        limits_ref: ds.limits_ref.clone(),
core/tests/golden_datasets.rs:67:                (ExpectedKind::Warnings, CompareMode::ReasonCodes) => {
core/tests/determinism_io.rs:26:        limits_ref: ds.limits_ref.clone(),
core/tests/determinism_wizard.rs:26:        limits_ref: ds.limits_ref.clone(),
core/part_ops/src/lib.rs:4:use craftcad_serialize::{Part, Polygon2D, Reason, ReasonCode, Result};
core/part_ops/src/lib.rs:14:        return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
core/part_ops/src/lib.rs:18:            return Err(Reason::from_code(ReasonCode::PartInvalidOutline));
core/tests/golden_harness_smoke.rs:30:        limits_ref: "default".to_string(),
core/tests/golden_harness_smoke.rs:104:        limits_ref: "default".to_string(),
core/src/testing/golden_harness.rs:16:    pub limits_ref: String,
core/src/testing/golden_harness.rs:33:    ReasonCodes,
core/src/testing/golden_harness.rs:101:        "dataset_id={} seed={} eps={} round={} ordering_tag={} limits_ref={} input_sha=[{}]",
core/src/testing/golden_harness.rs:107:        meta.limits_ref,
core/src/testing/golden_harness.rs:523:        (CompareKind::ReasonCodes, ActualData::Json(value)) => compare_reason_codes(
core/src/testing/golden_harness.rs:638:        kind: CompareKind::ReasonCodes,
core/src/testing/golden_harness.rs:650:            kind: CompareKind::ReasonCodes,
core/src/testing/golden_harness.rs:688:            kind: CompareKind::ReasonCodes,
core/src/testing/datasets_manifest.rs:17:    pub limits_ref: String,
core/src/testing/datasets_manifest.rs:87:    ReasonCodes,
core/src/testing/datasets_manifest.rs:179:    if ds.limits_ref.trim().is_empty() {
core/src/testing/datasets_manifest.rs:182:            "limits_ref must be non-empty",
core/src/testing/datasets_manifest.rs:185:        .with_field(format!("datasets[{idx}].limits_ref")));
core/faces/src/extract.rs:3:use craftcad_serialize::{Geom2D, Reason, ReasonCode, Result, Vec2};
core/faces/src/extract.rs:134:            return Err(Reason::from_code(ReasonCode::FaceNoClosedLoop));
core/faces/src/extract.rs:137:            return Err(Reason::from_code(ReasonCode::GeomInvalidNumeric));
core/faces/src/extract.rs:140:            let mut r = Reason::from_code(ReasonCode::FaceSelfIntersection);
core/faces/src/extract.rs:154:        return Err(Reason::from_code(ReasonCode::FaceNoClosedLoop));
core/faces/src/extract.rs:166:                    let mut r = Reason::from_code(ReasonCode::FaceAmbiguousLoop);
core/src/testing/determinism_harness.rs:119:        "limits_ref": meta.limits_ref,
```

## Tests executed
- cargo test --manifest-path core/Cargo.toml -p craftcad_diagnostics

## Allowlist self-check
- Allowed paths only: YES
- No deletions: YES

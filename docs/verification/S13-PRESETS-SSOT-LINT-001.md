# Verification: S13-PRESETS-SSOT-LINT-001

## Goal
Sprint13 PR1相当：Presets/Template/Library/DeterminismのSSOT追加と、spec_ssot_lintでのCIゲート化。

## Changed files
- `git diff --name-only`
```
core/serialize/Cargo.toml
core/serialize/tests/spec_ssot_lint.rs
docs/status/trace-index.json
```
- `git status --porcelain` (untracked含む)
```
 M core/serialize/Cargo.toml
 M core/serialize/tests/spec_ssot_lint.rs
 M docs/status/trace-index.json
?? .ci_logs/
?? docs/specs/determinism/
?? docs/specs/library/
?? docs/specs/presets/
?? docs/specs/templates/
```

## Commands & Evidence
### Preflight / History
- `git status --porcelain` => 初期状態は空（clean）
- `git log -n 30 --oneline` =>
```
ed95e78 Merge pull request #60 from teru1991/codex/improve-svg-import-quality-1rijgm
8923b6a Merge branch 'main' into codex/improve-svg-import-quality-1rijgm
e0e8b27 S12-IO-COMPAT-005: add compat policy, SSOT gates, and golden runner
cc5c10e Merge pull request #59 from teru1991/codex/improve-svg-import-quality-9e44y6
480dd45 Merge branch 'main' into codex/improve-svg-import-quality-9e44y6
6532b12 S12-IO-BRIDGE-004: unify shared IO pipeline and E2E bridge goldens
81c3028 Merge pull request #58 from teru1991/codex/improve-svg-import-quality-p3n330
c831364 Merge branch 'main' into codex/improve-svg-import-quality-p3n330
92a54e4 S12-IO-DXF-003: harden DXF entities parsing and export coverage
1196b37 Merge pull request #57 from teru1991/codex/improve-svg-import-quality
9d1b184 Improve SVG import determinism, limits, and path/transform coverage
e265644 Merge pull request #56 from teru1991/codex/implement-ssot-handling-for-i/o-extension
f3ab920 Implement SSOT-driven IO support and reason code parity
eaeecda 追加、修正
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
```
- `git branch -vv` =>
```
* feature/s13-presets-ssot-lint-001 ed95e78 Merge pull request #60 from teru1991/codex/improve-svg-import-quality-1rijgm
  work                              ed95e78 Merge pull request #60 from teru1991/codex/improve-svg-import-quality-1rijgm
```
- `git rev-parse HEAD` =>
```
ed95e78839cc77ac3e13a56b5ee7d097ea5eed53
```

### Lint / Tests
- `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library` =>
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/lib.rs (target/debug/deps/craftcad_serialize-103c8f709e090ccd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/reason_codes_lint.rs (target/debug/deps/reason_codes_lint-b6b13662045fe572)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/schema_lint.rs (target/debug/deps/schema_lint-c6133f2ffbfa730a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/schema_validation.rs (target/debug/deps/schema_validation-b573e0d40e22a3be)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

     Running tests/spec_ssot_lint.rs (target/debug/deps/spec_ssot_lint-8196b5f771b362dc)

running 1 test
test spec_ssot_lint_presets_templates_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.05s

```
- `cargo test` =>
```
; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/craftcad_geom2d-6964468d0afae3ec)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/math.rs (target/debug/deps/math-7104d99ea0bdcc83)

running 1 test
test round_step_handles_negative ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/craftcad_i18n-99c28a1f519980ee)

running 2 tests
test tests::formats_units ... ok
test tests::resolves_with_param ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/craftcad_io-00eddcb8d582d335)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/craftcad_io_bridge-469c8daef98e44b0)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/compat_matrix_golden.rs (target/debug/deps/compat_matrix_golden-72ac0f3e59744818)

running 1 test
test compat_report_golden ... FAILED

failures:

---- compat_report_golden stdout ----

thread 'compat_report_golden' (19077) panicked at crates/io_bridge/tests/compat_matrix_golden.rs:31:5:
assertion `left == right` failed: golden mismatch: /workspace/CraftCAD/core/crates/io_bridge/../../../tests/golden/io_roundtrip/expected/compat/compat_report.json
  left: "{\n  \"exports\": {\n    \"json_from_dxf_len\": 5512,\n    \"json_from_svg_len\": 2565\n  },\n  \"input_id\": \"compat_01\",\n  \"pipelines\": [\n    \"json->dxf->json\",\n    \"json->svg->json\"\n  ],\n  \"results\": [\n    {\n      \"deterministic\": true,\n      \"geometry_ok\": true,\n      \"notes\": [\n        \"bbox0=0.000000,0.000000,38.000000,20.000000\",\n        \"bbox1=0.000000,0.000000,38.000000,20.000000\"\n      ],\n      \"pipeline\": \"json->dxf->json\",\n      \"style_ok\": true,\n      \"warnings\": [\n        \"IO_TEXT_FALLBACK_FONT\",\n        \"IO_FALLBACK_024\"\n      ]\n    },\n    {\n      \"deterministic\": true,\n      \"geometry_ok\": true,\n      \"notes\": [\n        \"bbox0=0.000000,0.000000,38.000000,20.000000\",\n        \"bbox2=0.000000,0.000000,33.000000,14.000000\"\n      ],\n      \"pipeline\": \"json->svg->json\",\n      \"style_ok\": true,\n      \"warnings\": [\n        \"IO_UNIT_GUESSED\",\n        \"IO_TEXT_FALLBACK_FONT\"\n      ]\n    }\n  ],\n  \"schema_version\": 1\n}"
 right: "{\n  \"schema_version\": 1,\n  \"input_id\": \"compat_01\",\n  \"pipelines\": [\n    \"json->dxf->json\",\n    \"json->svg->json\"\n  ],\n  \"results\": [\n    {\n      \"pipeline\": \"json->dxf->json\",\n      \"deterministic\": true,\n      \"geometry_ok\": true,\n      \"style_ok\": true,\n      \"warnings\": [\n        \"IO_TEXT_FALLBACK_FONT\",\n        \"IO_FALLBACK_024\"\n      ],\n      \"notes\": [\n        \"bbox0=0.000000,0.000000,38.000000,20.000000\",\n        \"bbox1=0.000000,0.000000,38.000000,20.000000\"\n      ]\n    },\n    {\n      \"pipeline\": \"json->svg->json\",\n      \"deterministic\": true,\n      \"geometry_ok\": true,\n      \"style_ok\": true,\n      \"warnings\": [\n        \"IO_UNIT_GUESSED\",\n        \"IO_TEXT_FALLBACK_FONT\"\n      ],\n      \"notes\": [\n        \"bbox0=0.000000,0.000000,38.000000,20.000000\",\n        \"bbox2=0.000000,0.000000,33.000000,14.000000\"\n      ]\n    }\n  ],\n  \"exports\": {\n    \"json_from_dxf_len\": 5512,\n    \"json_from_svg_len\": 2565\n  }\n}"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    compat_report_golden

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p craftcad_io_bridge --test compat_matrix_golden`
```

## Spec alignment
- docs/specs/presets/ids_and_semver.md: ID/semver規約
- docs/specs/presets/compatibility.md: N-2/破壊変更ルール
- docs/specs/library/storage_layout.md: user dir/atomic write
- docs/specs/library/search_policy.md: scoring/tie-break
- docs/specs/determinism/wizard_policy.md: seed/順序/丸め
- docs/specs/presets/built_in_presets.json: 同梱資産
- docs/specs/templates/*.template.json: required_presets宣言

## Notes / Risks
- tags.schema.jsonはpolicy objectとして運用（Step3でreader実装）
- generation_stepsの解釈はStep4以降（このStepでは schema + required_presets 解決のみ）

## Allowlist self-check
- 変更が Allowed paths に収まっていること
- 削除が無いこと

## Commit evidence
- `git show --stat --oneline -1` =>
```
53108a9 Sprint13: add presets/templates/library SSOT and lint gate (step1)
 core/serialize/Cargo.toml                          |   1 +
 core/serialize/tests/spec_ssot_lint.rs             | 298 +++++++++++++++++++++
 docs/specs/determinism/wizard_policy.md            |   9 +
 docs/specs/library/search_policy.md                |  19 ++
 docs/specs/library/storage_layout.md               |  22 ++
 docs/specs/library/tags.schema.json                |  37 +++
 docs/specs/presets/built_in_presets.json           | 200 ++++++++++++++
 docs/specs/presets/compatibility.md                |   6 +
 docs/specs/presets/hardware_preset.schema.json     |  43 +++
 docs/specs/presets/ids_and_semver.md               |  20 ++
 docs/specs/presets/material_preset.schema.json     |  30 +++
 docs/specs/presets/output_preset.schema.json       |  30 +++
 docs/specs/presets/presets_bundle.schema.json      |  16 ++
 docs/specs/presets/process_preset.schema.json      |  33 +++
 docs/specs/templates/README.md                     |   8 +
 docs/specs/templates/box_wizard.template.json      |  24 ++
 .../templates/leather_pouch_wizard.template.json   |  24 ++
 docs/specs/templates/shelf_wizard.template.json    |  29 ++
 docs/specs/templates/wizard_template.schema.json   |  77 ++++++
 docs/status/trace-index.json                       |  14 +
 docs/verification/S13-PRESETS-SSOT-LINT-001.md     | 183 +++++++++++++
 21 files changed, 1123 insertions(+)
```

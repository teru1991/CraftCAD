# Verification: S13-WIZARDS-ENGINE-004

## Goal
テンプレ解釈エンジン（schema validate / deps解決 / 入力検証 / 安全DSL / 決定性seed）を完成させる。

## Changed files
- `git diff --name-only`
```
core/Cargo.toml
docs/status/trace-index.json
```

## Commands & Evidence
### Preflight / History
- `git status --porcelain`
```
 M core/Cargo.toml
 M docs/status/trace-index.json
?? core/crates/presets/
?? core/crates/wizards/
?? docs/verification/S13-WIZARDS-ENGINE-004.md
```
- `git log -n 30 --oneline`
```
245ea78 Sprint13: add presets/templates/library SSOT and lint gate (step1)
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
```
- `git branch -vv`
```
  feature/s13-presets-ssot-lint-001 245ea78 Sprint13: add presets/templates/library SSOT and lint gate (step1)
* feature/s13-wizards-engine-004    245ea78 Sprint13: add presets/templates/library SSOT and lint gate (step1)
  work                              ed95e78 Merge pull request #60 from teru1991/codex/improve-svg-import-quality-1rijgm
```
- `git rev-parse HEAD`
```
245ea782f278e24ad48f6a8d2c7d6381d483c852
```

### Tests
- `cargo test -p craftcad_wizards`
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Running unittests src/lib.rs (target/debug/deps/craftcad_wizards-3b41447a3c0b389e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism_seed.rs (target/debug/deps/determinism_seed-102d4060404cfafd)

running 1 test
test derived_seed_is_stable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/dsl_eval.rs (target/debug/deps/dsl_eval-ff86acc69bfbde80)

running 1 test
test eval_rejects_expressions_in_step4 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/input_validate.rs (target/debug/deps/input_validate-544a309e425dc91e)

running 1 test
test input_rejects_unknown_key ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/template_load.rs (target/debug/deps/template_load-f08260cd60249e15)

running 1 test
test load_templates_schema_ok ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests craftcad_wizards

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
- `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library`
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
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

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.04s

```
- `cargo test -q`（可能なら）
```
passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
compat_report_golden --- FAILED

failures:

---- compat_report_golden stdout ----

thread 'compat_report_golden' (28191) panicked at crates/io_bridge/tests/compat_matrix_golden.rs:31:5:
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
- wizard_template.schema.json 検証
- required_presets 解決（missing時はWizardDepMissingPreset）
- wizard_policy.md: seed順位（explicit > derived）、順序固定（sort）
- 任意コード実行禁止：w_expr/h_expr等はStep4では拒否

## Notes / Risks
- Boxテンプレの式はStep5で “安全な式評価” を追加して解禁する（現Stepは安全最優先で拒否）
- Step5で Part生成（図形/穴/注釈）へ接続する

## Allowlist self-check
- Allowed paths内のみ
- 削除なし

## Commit evidence
- `git show --stat --oneline -1`
```
183c4af Sprint13: add wizards template engine with safe DSL and determinism (step4)
 core/Cargo.toml                               |   2 +
 core/crates/presets/Cargo.toml                |  11 ++
 core/crates/presets/src/lib.rs                | 136 +++++++++++++++
 core/crates/wizards/Cargo.toml                |  22 +++
 core/crates/wizards/src/determinism.rs        |  35 ++++
 core/crates/wizards/src/engine/ast.rs         |  50 ++++++
 core/crates/wizards/src/engine/eval.rs        | 237 +++++++++++++++++++++++++
 core/crates/wizards/src/engine/mod.rs         |   3 +
 core/crates/wizards/src/engine/validate.rs    | 160 +++++++++++++++++
 core/crates/wizards/src/lib.rs                | 121 +++++++++++++
 core/crates/wizards/src/reasons.rs            |  39 +++++
 core/crates/wizards/src/template.rs           | 176 +++++++++++++++++++
 core/crates/wizards/src/types.rs              |  33 ++++
 core/crates/wizards/tests/determinism_seed.rs |  52 ++++++
 core/crates/wizards/tests/dsl_eval.rs         |  43 +++++
 core/crates/wizards/tests/input_validate.rs   |  19 ++
 core/crates/wizards/tests/template_load.rs    |  10 ++
 docs/status/trace-index.json                  |  11 ++
 docs/verification/S13-WIZARDS-ENGINE-004.md   | 242 ++++++++++++++++++++++++++
 19 files changed, 1402 insertions(+)
```

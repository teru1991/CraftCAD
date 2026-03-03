# S15-STEP4 Verification (binary-free)

## History evidence
- `git status --porcelain`
  - M core/crates/ssot_lint/Cargo.toml
  - M core/src/testing/mod.rs
  - M docs/status/trace-index.json
  - ?? core/crates/ssot_lint/tests/compat_io_read.rs
  - ?? core/crates/ssot_lint/tests/compat_presets_read.rs
  - ?? core/crates/ssot_lint/tests/compat_projects_read.rs
  - ?? core/crates/ssot_lint/tests/compat_templates_read.rs
  - ?? core/src/testing/compat_harness.rs
  - ?? core/tests/compat_io_read.rs
  - ?? core/tests/compat_presets_read.rs
  - ?? core/tests/compat_projects_read.rs
  - ?? core/tests/compat_templates_read.rs
  - ?? scripts/testing/run_compat.sh
  - ?? tests/compat/README.md
  - ?? tests/compat/io/external_rect.svg
  - ?? tests/compat/presets/n-2_builtin_preset.json
  - ?? tests/compat/projects/forward_incompatible_project.json
  - ?? tests/compat/projects/n-2_large_project.json
  - ?? tests/compat/projects/n-2_small_project.json
  - ?? tests/compat/templates/n-2_shelf_template.json
- `git log --oneline -n 20`
  - b6f7e82 S15-STEP3: expand binary-free golden smoke datasets
  - 6588498 Merge pull request #76 ...
- `git log --graph --decorate --oneline --all -n 40`
  - branch start: `feature/s15-step4-001` from `b6f7e82`
- `git branch -vv`
  - `* feature/s15-step4-001 b6f7e82 ...`
- `git rev-parse HEAD`
  - `b6f7e82363f19027691c411536afbe677033d3bd`

## What / Why
- `tests/compat/**` に N-2互換の証拠資産（テキストのみ）を固定。
- `core/src/testing/compat_harness.rs` を追加し、以下を実APIで検証:
  - project(JSON): `JsonIo` import/export で open→(migrate path算出)→validate
  - preset(JSON): `PresetsService` + `PresetRef` resolve
  - template(JSON): `Template` deserialize + `validate_inputs` + `eval_generation_steps`(dry-run)
  - io(SVG/DXF): `SvgIo` / `DxfIo` import + normalize/export
- forward_incompatible project は `CP_FORWARD_INCOMPATIBLE` を返し、クラッシュしないことをテストで固定。
- 失敗時 artifact は `failure_artifacts/compat/<case_id>/` に text-only で出力（.bin 不使用）。

## Assets added (no delete, text-only)
- tests/compat/projects/n-2_small_project.json
- tests/compat/projects/n-2_large_project.json
- tests/compat/projects/forward_incompatible_project.json
- tests/compat/presets/n-2_builtin_preset.json
- tests/compat/templates/n-2_shelf_template.json
- tests/compat/io/external_rect.svg

## Binary-free check
- `find tests/compat/projects tests/compat/presets tests/compat/templates tests/compat/io -type f ! -name '*.json' ! -name '*.svg' ! -name '*.dxf' -print`
  - output: empty
- UTF-8/NUL check (python)
  - output: `compat fixture text check: ok`

## How verified
- `cargo test -p core` (repo rootにCargo.tomlがないため失敗)
- `./scripts/testing/run_compat.sh`
- `cargo test --manifest-path core/Cargo.toml -p ssot_lint`

## Sample failure_artifacts tree
- Forward-incompatible case (`project_forward_incompat`) で生成される出力例:
  - `failure_artifacts/compat/project_forward_incompat/meta.json`
  - `failure_artifacts/compat/project_forward_incompat/repro_input.json`
  - `failure_artifacts/compat/project_forward_incompat/actual.json`
  - `failure_artifacts/compat/project_forward_incompat/diff.txt`
  - `failure_artifacts/compat/project_forward_incompat/reason_codes.json`

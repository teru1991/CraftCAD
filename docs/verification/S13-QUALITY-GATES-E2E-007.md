# Verification: S13-QUALITY-GATES-E2E-007

## Goal
Sprint13 DoDの品質ゲート（Golden/Determinism/Compat/E2E）をCIで必須化し、棚板フローE2Eを固定する。

## Changed files
- `.github/workflows/rust-ci.yml`
- `core/crates/wizards/Cargo.toml`
- `scripts/ci/run_all.sh`
- `tests/e2e/flow_shelf_to_nest_to_export.rs`
- `tests/determinism/wizard_determinism.rs`
- `tests/compat/presets_templates_compat.rs`
- `docs/status/trace-index.json`
- `docs/verification/S13-QUALITY-GATES-E2E-007.md`

## Commands & Evidence
### Preflight / History
- `git status --porcelain` (clean at start)
- `git fetch --all --prune`
- `git checkout -b feature/s13-quality-gates-e2e-007`
- `git log -n 50 --oneline` (HEAD was `1c41875`)
- `git branch -vv`
- `git rev-parse HEAD` (`1c41875b1083586c6a0989ebc3a72eec4804fb87`)

### Tests
- `cargo test -p craftcad_serialize --test spec_ssot_lint`
  - PASS (`spec_ssot_lint_presets_templates_library` を含む9件成功)
- `cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export -- --nocapture`
  - 初回FAILで hash を取得: `c5a67b26e2b17dd8a934189b0716ae782bd3cffe947bf5459a3c482a8859e70e`
  - expected更新後、再実行PASS
- `cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export`
  - PASS
- `cargo test -p craftcad_wizards --test wizard_determinism`
  - PASS（10-run一致）
- `cargo test -p craftcad_wizards --test presets_templates_compat`
  - 初回は型不一致でFAIL（`PathBuf`修正）
  - 修正後PASS
- `cargo test --workspace --all-targets`
  - FAIL（既知）: `craftcad_io_bridge` の `compat_matrix_golden` で JSON key ordering mismatch


### Commit stat
- `git show --stat --oneline HEAD`
  - `.github/workflows/rust-ci.yml | 9 +++`
  - `core/crates/wizards/Cargo.toml | 13 ++++`
  - `docs/status/trace-index.json | 15 +++++`
  - `docs/verification/S13-QUALITY-GATES-E2E-007.md | 62 +++++++++++++++++++`
  - `scripts/ci/run_all.sh | 3 +`
  - `tests/compat/presets_templates_compat.rs | 36 +++++++++++`
  - `tests/determinism/wizard_determinism.rs | 69 +++++++++++++++++++++`
  - `tests/e2e/flow_shelf_to_nest_to_export.rs | 83 ++++++++++++++++++++++++++`

### CI/workflow gate wiring
- `scripts/ci/run_all.sh` に以下を追加
  - `cargo test -p craftcad_wizards --test flow_shelf_to_nest_to_export`
  - `cargo test -p craftcad_wizards --test wizard_determinism`
  - `cargo test -p craftcad_wizards --test presets_templates_compat`
- `.github/workflows/rust-ci.yml` の `core-rust-ci` job に同3ゲートを追加

## Gate status
- SSOT lint: PASS
- presets/library/wizards: PASS（対象追加分）
- determinism: PASS（10-run一致）
- compat: PASS（builtin presets/templates load）
- e2e: PASS（hash固定）

## Notes / Risks
- nesting本体が未実装でも `recommended_nest_job` までは生成するため導線が成立
- 出力は internal json を主ゲートにし、svg/dxfはSprint12の実装状況次第で別ゲート化
- workspace全体は既知の `compat_matrix_golden` 並び順差分で依然FAIL（今回変更外）

## Allowlist self-check
- Allowed paths内のみ変更
- 削除なし

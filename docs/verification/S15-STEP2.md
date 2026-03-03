# S15-STEP2 Verification

## History evidence
### Preflight commands and key outputs
- `git status --porcelain`
  - (no output; clean before Step2)
- `git fetch --all --prune`
  - completed without error
- `git checkout -b feature/s15-step2-001`
  - `Switched to a new branch 'feature/s15-step2-001'`
- `git log --oneline -n 20`
  - `d375c48 S15-STEP1: add datasets manifest SSOT schema and lint gate`
  - `8bf61bd Merge pull request #74 ...`
- `git log --graph --decorate --oneline --all -n 40`
  - branch head at start: `feature/s15-step2-001`
- `git branch -vv`
  - `* feature/s15-step2-001 d375c48 ...`
- `git rev-parse HEAD`
  - `d375c4830caab0af1cd5ee334d21a277ada2583c`

## Baseline
- `cargo test -p core` @ repo root: failed (no Cargo.toml at repository root)
- `cargo test -p core` @ `/workspace/CraftCAD/core`: failed (package `core` does not exist in workspace)
- Practical gate command for this repository topology: `cargo test -p ssot_lint`

## What / Why
- Golden比較を共通ハーネスに集約し、JSON/SVG/bytes/ReasonCode の正規化・比較・差分を一箇所で扱うようにした。
- 失敗時は `failure_artifacts/<dataset_id>/` に再現情報と差分を出力する実装を追加した。

## Spec alignment
- `docs/specs/testing/golden_policy.md` を追加し、以下を固定:
  - JSON key sort + round_step量子化 + NaN/Inf拒否
  - SVGの保守的正規化（属性ソート、数値丸め、空白正規化）
  - bytes比較の位置付け（最後の手段）
  - ReasonCode を dedupe + stable sort
  - failure artifacts への必須出力項目

## Changed files (allowlist)
- `docs/specs/testing/golden_policy.md`
- `tests/golden/README.md`
- `core/src/testing/golden_harness.rs`
- `core/src/testing/mod.rs`
- `core/tests/golden_harness_smoke.rs`
- `core/tests/golden_harness_svg_normalize.rs`
- `core/crates/ssot_lint/Cargo.toml`
- `core/crates/ssot_lint/tests/golden_harness_smoke.rs`
- `core/crates/ssot_lint/tests/golden_harness_svg_normalize.rs`
- `docs/verification/S15-STEP2.md`
- `docs/status/trace-index.json`

## How verified
- `cargo test -p ssot_lint` ✅
- `./scripts/ci/run_all.sh` (first run) ❌ rust_fmt fail
- `.ci_logs/summary.json` で rust_fmt失敗を確認 ✅
- `cargo fmt --all --manifest-path core/Cargo.toml` ✅
- `./scripts/ci/run_all.sh` (rerun) ✅
- `.ci_logs/summary.json` 最終結果 `total_failures: 0` ✅

## Determinism notes / limitations
- JSON正規化は BTreeMap 経由で key 順序を固定。配列は `OrderingPolicy::Strict` で入力順維持。
- SVG正規化は外部parserを使わない保守的実装。一般的なタグ/属性形式は正規化し、失敗時は whitespace-only 正規化へ決定的にフォールバック。
- Artifacts 出力先は `CRAFTCAD_FAILURE_ARTIFACTS_DIR` で固定可能（CI artifact upload と相性が良い）。


## Follow-up fix
- GoldenMismatch message payload now always includes reproducibility context from dataset meta, including `input_sha` derived from `inputs[].sha256` (or `-` when absent), alongside dataset_id/seed/eps/round/ordering_tag/limits_ref.

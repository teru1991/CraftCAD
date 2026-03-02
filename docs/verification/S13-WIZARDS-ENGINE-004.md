# Verification: S13-WIZARDS-ENGINE-004

## Goal
テンプレ解釈エンジン（schema validate / deps解決 / 入力検証 / 安全DSL / 決定性seed）を完成させる。

## Changed files
- `docs/status/trace-index.json`
- `docs/verification/S13-WIZARDS-ENGINE-004.md`

## Commands & Evidence
### Preflight / History
- `git status --porcelain`
- `git fetch --all --prune`
- `git checkout -b feature/s13-wizards-engine-004`
- `git log -n 30 --oneline`
- `git branch -vv`
- `git rev-parse HEAD` → `f9172d50473a5dfac889f57386dded61aca77dd8`
- `cargo test -q` (repo root) → fail (`Cargo.toml` not found at repo root; workspace is under `core/`)

### Tests
- `cargo test -p craftcad_wizards` (in `core/`) → pass
- `cargo test` (in `core/`) → fail at existing `craftcad_io_bridge/tests/compat_matrix_golden.rs::compat_report_golden` (known unrelated failure)

## Spec alignment
- `wizard_template.schema.json` によるテンプレ検証。
- `required_presets` 解決（missing時は `WizardDepMissingPreset`）。
- `wizard_policy.md`: seed順位（explicit > derived）、順序固定（ソート）。
- 任意コード実行禁止：`w_expr/h_expr` 等は Step4 では拒否。

## Notes / Risks
- Boxテンプレの式はStep5で “安全な式評価” を追加して解禁する（現Stepは安全最優先で拒否）。
- Step5で Part生成（図形/穴/注釈）へ接続する。

## Allowlist self-check
- Allowed paths内のみ。
- 削除なし。

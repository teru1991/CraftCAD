# Wizard Templates SSOT

- `template_id` / `template_version` / `schema_version` は、テンプレート識別子・SemVer・スキーマ互換単位を表す。
- `required_presets` はテンプレートが必要とする built-in/user preset ID を種別ごとに宣言する。
- `ui_inputs` は `unit` / `min` / `max` / `step` を含む入力制約を宣言する。
- `generation_steps` は宣言的に記述し、任意コード実行は禁止。
- `determinism.seed_source` は seed決定元（詳細は `docs/specs/determinism/wizard_policy.md`）を示す。
- ローカルlint実行コマンド: `cargo test -p craftcad_serialize spec_ssot_lint_presets_templates_library`

# Verification: S13-LIBRARY-CRATE-003

## Goal
libraryクレート（tags/index/deps/store）を決定性・破損耐性つきで完成させ、unitテストで担保する。

## Changed files
- `core/Cargo.toml`
- `core/crates/library/Cargo.toml`
- `core/crates/library/src/lib.rs`
- `core/crates/library/src/tags.rs`
- `core/crates/library/src/index.rs`
- `core/crates/library/src/store.rs`
- `core/crates/library/src/deps.rs`
- `core/crates/library/src/reasons.rs`
- `core/crates/library/tests/tags_policy.rs`
- `core/crates/library/tests/index_search.rs`
- `core/crates/library/tests/deps_resolve.rs`
- `core/crates/library/tests/store_rebuild.rs`
- `docs/status/trace-index.json`
- `docs/verification/S13-LIBRARY-CRATE-003.md`

## Commands & Evidence
### Preflight / History
- `git status --porcelain` → clean at start, modified files after implementation.
- `git fetch --all --prune` → success.
- `git checkout -b feature/s13-library-crate-003` → branch created.
- `git log -n 30 --oneline` → history captured.
- `git branch -vv` → branch pointers captured.
- `git rev-parse HEAD` → `ba15546708a4b1b97307719e509afdfe4608dc0f` (pre-change).

### Tests
- `cargo test -q` (repo root) → failed: Cargo.toml not found at repo root (workspace is under `core/`).
- `cargo test -p craftcad_library` (in `core/`) → pass.
- `cargo test` (in `core/`) → fail at existing test `craftcad_io_bridge/tests/compat_matrix_golden.rs` (`compat_report_golden` ordering mismatch), unrelated to this task.

## Spec alignment
- `docs/specs/library/tags.schema.json`: policy準拠（normalize/forbidden/max_len）。
- `docs/specs/library/search_policy.md`: scoring + tie-breakの決定性。
- `docs/specs/library/storage_layout.md`: index保存場所/再構築可能。
- `docs/specs/templates/*.template.json`: required_presets 解決。

## Notes / Risks
- Templateの完全解釈はStep4（wizards engine）で実装。本Stepは依存解決と検索基盤に集中。
- indexは postings list を stable sort/dedup し、search結果も決定的にソート。

### Post-commit
- `git show --stat --oneline -1` → latest commit includes 14 files changed (library crate + verification/trace).

## Allowlist self-check
- Allowed paths内のみ。
- 削除なし。

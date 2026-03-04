# Verification: S17-DIAG-RET-FP-004

## Goal
- Retention（保持/削除/容量上限）を完成させ、SSOT Fingerprint（固定リストsha256）を本実装し、SupportZipBuilderに統合する。

## Changed files
- core/crates/diagnostics/src/ssot_fingerprint.rs (edit: full impl)
- core/crates/diagnostics/src/retention.rs (edit: validation)
- core/crates/diagnostics/src/store.rs (new)
- core/crates/diagnostics/src/support_zip.rs (edit: build_in_dir + build_into_store + fingerprint required)
- core/crates/diagnostics/src/lib.rs (edit: export store/support_zip)
- core/crates/diagnostics/Cargo.toml (edit: dev-dependency tempfile)
- core/crates/diagnostics/tests/retention_fp_basic.rs (new)
- core/crates/diagnostics/tests/ssot_fingerprint_list.rs (new)
- docs/status/trace-index.json (edit; tasks["S17-DIAG-RET-FP-004"] only)

## Determinism
- fingerprint items: path sort
- cleanup deletion order: created_at asc -> id asc

## Privacy
- index.jsonl: relative path only
- fingerprint: path is repo-relative only（PIIなし）

## History evidence (paste outputs)
- rg -n "retention_policy|cleanup|ssot_fingerprint_paths" -S core/crates/diagnostics
```text
core/crates/diagnostics/src/store.rs:106:    pub fn cleanup(&self, policy: &RetentionPolicy) -> io::Result<CleanupResult> {
core/crates/diagnostics/src/ssot_fingerprint.rs:62:pub fn ssot_fingerprint_paths() -> Vec<PathBuf> {
```

- ls -la docs/specs
```text
total 108
drwxr-xr-x 27 root root 4096 Mar  3 12:16 .
drwxr-xr-x  8 root root 4096 Mar  3 12:16 ..
drwxr-xr-x  2 root root 4096 Mar  3 12:16 cad
drwxr-xr-x  2 root root 4096 Mar  3 12:16 determinism
drwxr-xr-x  2 root root 4096 Mar  3 23:11 diagnostics
drwxr-xr-x  2 root root 4096 Mar  3 12:16 diycad
drwxr-xr-x  2 root root 4096 Mar  3 12:16 drawing
drwxr-xr-x  2 root root 4096 Mar  3 12:16 drawing_style
drwxr-xr-x  2 root root 4096 Mar  3 12:16 errors
drwxr-xr-x  2 root root 4096 Mar  3 12:16 export
drwxr-xr-x  2 root root 4096 Mar  3 12:16 ffi
drwxr-xr-x  2 root root 4096 Mar  3 12:16 io
drwxr-xr-x  2 root root 4096 Mar  3 12:16 library
drwxr-xr-x  2 root root 4096 Mar  3 12:16 perf
drwxr-xr-x  2 root root 4096 Mar  3 12:16 presets
drwxr-xr-x  2 root root 4096 Mar  3 12:16 project_file
drwxr-xr-x  2 root root 4096 Mar  3 12:16 reasons
drwxr-xr-x  2 root root 4096 Mar  3 12:16 recovery
drwxr-xr-x  2 root root 4096 Mar  3 12:16 schema
drwxr-xr-x  2 root root 4096 Mar  3 12:16 security
drwxr-xr-x  2 root root 4096 Mar  3 12:16 templates
drwxr-xr-x  2 root root 4096 Mar  3 12:16 testing
drwxr-xr-x  2 root root 4096 Mar  3 12:16 ui
drwxr-xr-x  2 root root 4096 Mar  3 12:16 units
drwxr-xr-x  2 root root 4096 Mar  3 12:16 vectors
drwxr-xr-x  2 root root 4096 Mar  3 12:16 wizard
```

## Tests executed
- cargo test -p craftcad_diagnostics

## Allowlist self-check
- Allowed paths only: YES
- No deletions: YES

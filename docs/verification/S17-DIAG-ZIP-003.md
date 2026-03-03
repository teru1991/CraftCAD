Verification: S17-DIAG-ZIP-003

## Goal
- SupportZipBuilder を実装し、atomic生成・limits・sha256・固定レイアウト・consent反映を満たす。

## Changed files
- core/crates/diagnostics/Cargo.toml (edit: zip/tempfile)
- core/crates/diagnostics/src/support_zip.rs (new)
- core/crates/diagnostics/src/lib.rs (edit: export)
- core/crates/diagnostics/tests/support_zip_basic.rs (new)
- (optional) docs/status/trace-index.json (edit; tasks["S17-DIAG-ZIP-003"] only)

## What / Why
- ユーザーが安全に提出できる“診断ZIP”を、クラッシュせずに生成する製品品質の核を作る。
- ZIPは契約(support_zip.md)に従い、同意が無い限りプロジェクト/入力コピーを含めない。

## Security / Privacy
- ZIP内のパスは固定＆sanitize（..や絶対パス禁止）
- 任意添付は consent true のときのみ
- limits超過は除外/トランケート＋ DIAG_ZIP_TRUNCATED warnings

## History evidence (paste outputs)
- git log -n 25 --oneline

```
d31bf30 Add diagnostics crate with JobLog/OpLog, SSOT schemas, redaction/limits and tests
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

- rg -n "support_zip|zip::ZipWriter|atomic|sha256" -S core

```
core/src/testing/golden_harness.rs:25:    pub sha256: Option<String>,
core/src/testing/golden_harness.rs:95:                i.sha256.as_deref().unwrap_or("-")
core/src/testing/golden_harness.rs:768:            &format!("expected_sha256={expected_hash}"),
core/src/testing/golden_harness.rs:769:            &format!("actual_sha256={actual_hash}"),
core/src/testing/determinism_harness.rs:12:    pub model_sha256: String,
core/src/testing/determinism_harness.rs:48:    let model_sha256 = hash_bytes(&bytes);
core/src/testing/determinism_harness.rs:55:        extra.insert("exported_svg_sha256".to_string(), hash_bytes(s.as_bytes()));
core/src/testing/determinism_harness.rs:61:        extra.insert("exported_json_sha256".to_string(), hash_bytes(&jb));
core/src/testing/determinism_harness.rs:65:        model_sha256,
core/src/testing/datasets_manifest.rs:46:    pub sha256: Option<String>,
core/src/testing/datasets_manifest.rs:272:    if let Some(hash) = &input.sha256 {
core/src/testing/datasets_manifest.rs:276:                "sha256 must be 64 lowercase hex chars",
core/src/testing/datasets_manifest.rs:278:            .with_field("inputs[].sha256"));
core/ffi_desktop/src/lib.rs:47:use std::sync::atomic::{AtomicU64, Ordering};
core/ffi_desktop/tests/ffi_contract_lint.rs:15:        .find(|l| l.contains("ffi_symbols_sha256"))
core/crates/library/src/index.rs:43:fn sha256_hex(s: &str) -> String {
core/crates/library/src/index.rs:54:        sha256_hex(&format!("{version}:{source}"))
core/crates/library/src/store.rs:31:pub fn save_index_atomic(
core/crates/library/src/store.rs:121:    let _ = save_index_atomic(layout, &idx);
core/crates/security/src/consent.rs:6:    pub support_zip_include_project: bool,
core/crates/wizards/src/run_record.rs:3:pub fn sha256_hex(s: &str) -> String {
core/crates/wizards/src/run_record.rs:11:    sha256_hex(&s)[..16].to_string()
core/crates/wizards/src/determinism.rs:6:fn sha256_u64(bytes: &[u8]) -> u64 {
core/crates/wizards/src/determinism.rs:21:    SSOT_DEFAULT_SEED ^ sha256_u64(s.as_bytes())
core/crates/ssot_lint/tests/determinism_io.rs:44:                sha256: i.sha256.clone(),
core/crates/ssot_lint/tests/golden_harness_smoke.rs:38:            sha256: None,
core/crates/ssot_lint/tests/golden_datasets.rs:55:                sha256: i.sha256.clone(),
core/crates/ssot_lint/tests/determinism_wizard.rs:44:                sha256: i.sha256.clone(),
core/crates/ssot_lint/tests/determinism_migrate.rs:44:                sha256: i.sha256.clone(),
core/crates/diagnostics/src/lib.rs:9:pub mod support_zip;
core/crates/diagnostics/src/lib.rs:20:pub use support_zip::{SupportZipBuilder, ZipResult as SupportZipResult, ZipWarnings};
core/crates/diagnostics/src/support_zip.rs:41:    pub sha256: String,
core/crates/diagnostics/src/support_zip.rs:155:            let mut zw = zip::ZipWriter::new(&mut tmp);
core/crates/diagnostics/src/support_zip.rs:288:        let (sha256, size_bytes) = compute_sha256_and_size(&mut tmp_file)?;
core/crates/diagnostics/src/support_zip.rs:296:            sha256,
core/crates/diagnostics/src/support_zip.rs:312:        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
core/crates/diagnostics/src/support_zip.rs:343:        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
core/crates/diagnostics/src/support_zip.rs:377:        zw: &mut zip::ZipWriter<&mut NamedTempFile>,
core/crates/diagnostics/src/support_zip.rs:415:fn compute_sha256_and_size(f: &mut File) -> io::Result<(String, u64)> {
core/crates/diagnostics/src/joblog.rs:49:    pub sha256: String,
core/crates/diagnostics/src/joblog.rs:102:    pub sha256: String,
core/crates/diagnostics/src/joblog.rs:217:        sha256: &str,
core/crates/diagnostics/src/joblog.rs:228:        let (sha, _) = truncate_str(self.redactor.redact_str(sha256), 64);
core/crates/diagnostics/src/joblog.rs:236:            sha256: sha,
core/crates/diagnostics/src/joblog.rs:300:    pub fn attach_exported_file(&mut self, format: &str, sha256: &str, size_bytes: i64) {
core/crates/diagnostics/src/joblog.rs:314:        let (sha, _) = truncate_str(self.redactor.redact_str(sha256), 64);
core/crates/diagnostics/src/joblog.rs:319:            sha256: sha,
core/crates/diagnostics/src/ssot_fingerprint.rs:12:    pub sha256: String,
core/crates/diagnostics/src/repro.rs:9:    pub zip_sha256: String,
core/crates/diagnostics/src/repro.rs:56:            .then_with(|| a.sha256.cmp(&b.sha256))
core/crates/diagnostics/src/repro.rs:61:            "- kind: {}, sha256: {}, size_bytes: {}",
core/crates/diagnostics/src/repro.rs:62:            i.kind, i.sha256, i.size_bytes
core/crates/diagnostics/src/repro.rs:98:        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: {}", a.zip_sha256);
core/crates/diagnostics/src/repro.rs:101:        let _ = writeln!(&mut s, "- diagnostics_zip_sha256: (n/a)");
core/crates/diagnostics/src/oplog.rs:13:fn sha256_hex(bytes: &[u8]) -> String {
core/crates/diagnostics/src/oplog.rs:96:        let params_hash = sha256_hex(&s);
core/crates/diagnostics/tests/support_zip_basic.rs:30:fn support_zip_contains_required_entries_and_respects_consent_false() {
core/crates/diagnostics/tests/support_zip_basic.rs:75:    assert_eq!(res.sha256.len(), 64);
core/crates/jobs/src/cancel.rs:6:use std::sync::atomic::{AtomicBool, Ordering};
core/diag/src/lib.rs:46:    let mut zip = zip::ZipWriter::new(cursor);
core/crates/cache/src/key.rs:25:        CanonicalJson::new(v).sha256_hex()
core/crates/cache/src/key.rs:45:    pub fn sha256_hex(&self) -> Sha256Hex {
core/crates/cache/src/key.rs:80:    /// Input payload hash (sha256 hex).
core/crates/cache/src/key.rs:81:    pub inputs_sha256: String,
core/crates/cache/src/key.rs:82:    /// Options hash (sha256 hex).
core/crates/cache/src/key.rs:83:    pub options_sha256: String,
core/crates/cache/src/key.rs:84:    /// SSOT contract hash (sha256 hex).
core/crates/cache/src/key.rs:102:            ("inputs_sha256", &self.inputs_sha256),
core/crates/cache/src/key.rs:103:            ("options_sha256", &self.options_sha256),
core/crates/cache/src/key.rs:123:            "inputs_sha256": self.inputs_sha256,
core/crates/cache/src/key.rs:124:            "options_sha256": self.options_sha256,
core/crates/cache/src/key.rs:144:            hash: canon.sha256_hex(),
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:12:        inputs_sha256: inputs.to_string(),
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:13:        options_sha256: opts.to_string(),
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:32:    m3.inputs_sha256 = "eeeeeeee".to_string();
core/crates/cache/tests/cache_lru_determinism_invalidation.rs:37:    m4.options_sha256 = "ffffffff".to_string();
core/serialize/tests/spec_ssot_lint.rs:1150:        "support_zip.md",
core/serialize/tests/spec_ssot_lint.rs:1199:    let zip_md = read_text_path(&dir.join("support_zip.md"));
core/serialize/tests/spec_ssot_lint.rs:1202:        "missing required marker in support_zip.md: needle=joblog.json"
core/serialize/tests/spec_ssot_lint.rs:1206:        "missing required marker in support_zip.md: needle=reason_summary.json"
core/serialize/tests/spec_ssot_lint.rs:1210:        "missing required marker in support_zip.md: needle=ssot_fingerprint.json"
core/crates/diycad_format/src/save.rs:59:fn write_zip_atomic(target: &Path, tmp: &Path) -> Result<()> {
core/crates/diycad_format/src/save.rs:180:    let mut zipw = zip::ZipWriter::new(tmp_file);
core/crates/diycad_format/src/save.rs:219:    if opt.atomic {
core/crates/diycad_format/src/save.rs:220:        write_zip_atomic(path, &tmp_path)?;
core/crates/diycad_format/src/types.rs:54:    pub sha256: String,
core/crates/diycad_format/src/types.rs:165:    pub atomic: bool,
core/crates/diycad_format/src/types.rs:175:            atomic: true,
core/tests/determinism_io.rs:33:                sha256: i.sha256.clone(),
core/crates/diycad_format/src/integrity.rs:7:fn sha256_hex(bytes: &[u8]) -> String {
core/crates/diycad_format/src/integrity.rs:19:            sha256: sha256_hex(bytes),
core/crates/diycad_format/src/integrity.rs:73:        if sha256_hex(&bytes) != e.sha256 {
core/crates/diycad_format/src/integrity.rs:79:                message: "sha256 mismatch".to_string(),
core/tests/golden_harness_smoke.rs:34:            sha256: None,
core/tests/golden_datasets.rs:41:                sha256: i.sha256.clone(),
core/crates/diycad_format/tests/salvage_basic.rs:11:    let mut zw = zip::ZipWriter::new(f);
core/crates/diycad_format/tests/limits_zipbomb.rs:11:    let mut zw = zip::ZipWriter::new(f);
core/tests/determinism_wizard.rs:33:                sha256: i.sha256.clone(),
core/crates/diycad_format/tests/atomic_save.rs:34:fn atomic_save_does_not_leave_missing_target() {
core/tests/determinism_migrate.rs:33:                sha256: i.sha256.clone(),
core/crates/sketch/src/id.rs:1:use std::sync::atomic::{AtomicU64, Ordering};
```

## Tests executed
- cargo test --manifest-path core/Cargo.toml -p craftcad_diagnostics

## Allowlist self-check
- Allowed paths only: YES
- No deletions: YES

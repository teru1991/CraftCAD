# Verification: S17-DIAG-UX-005

## Goal
- UI未完成でも “自己解決→提出” を最短で実行できる CLI/導線を用意し、将来のSupport Dialogに接続しやすいAPIを固定する。

## Changed files
- core/crates/diagnostics/src/cli.rs (new)
- core/crates/diagnostics/examples/diagnostics_cli.rs (new)
- core/crates/diagnostics/src/lib.rs (edit: pub mod cli)
- core/crates/diagnostics/Cargo.toml (edit: clap/anyhow)
- scripts/diagnostics/run_support_zip.sh (new)
- scripts/diagnostics/run_support_zip.ps1 (new)
- core/crates/diagnostics/tests/cli_smoke.rs (new)
- docs/status/trace-index.json (edit; tasks["S17-DIAG-UX-005"] only)

## Privacy / Consent
- include_project/include_inputs は default false
- preview で ZIP構成と同意状態を明示

## History evidence (paste outputs)
- ls -la apps apps/desktop core/ffi_desktop/src
```text
apps:
total 16
drwxr-xr-x  4 root root 4096 Mar  3 12:16 .
drwxr-xr-x 14 root root 4096 Mar  3 12:16 ..
drwxr-xr-x  3 root root 4096 Mar  3 12:16 desktop
drwxr-xr-x  4 root root 4096 Mar  3 12:16 mobile

apps/desktop:
total 20
drwxr-xr-x 3 root root 4096 Mar  3 12:16 .
drwxr-xr-x 4 root root 4096 Mar  3 12:16 ..
-rw-r--r-- 1 root root    0 Mar  3 12:16 .gitkeep
-rw-r--r-- 1 root root 1532 Mar  3 12:16 CMakeLists.txt
-rw-r--r-- 1 root root 1763 Mar  3 12:16 README.md
drwxr-xr-x 5 root root 4096 Mar  3 12:16 src

core/ffi_desktop/src:
total 60
drwxr-xr-x 2 root root  4096 Mar  3 12:16 .
drwxr-xr-x 5 root root  4096 Mar  3 12:16 ..
-rw-r--r-- 1 root root   708 Mar  3 12:16 editor_bridge.rs
-rw-r--r-- 1 root root 44434 Mar  3 12:16 lib.rs
-rw-r--r-- 1 root root   477 Mar  3 12:16 perf_bridge.rs
```

- rg -n "ffi_desktop|bridge|cli|clap" -S core apps scripts
```text
scripts/diagnostics/run_support_zip.sh:5:cargo run -p craftcad_diagnostics --example diagnostics_cli --manifest-path core/Cargo.toml -- zip --out "$OUT"
scripts/diagnostics/run_support_zip.ps1:6:cargo run -p craftcad_diagnostics --example diagnostics_cli --manifest-path core/Cargo.toml -- zip --out $Out
core/crates/diagnostics/Cargo.toml:17:clap = { version = "4", features = ["derive"] }
core/crates/diagnostics/src/cli.rs:2:use clap::{Parser, Subcommand};
core/crates/diagnostics/src/lib.rs:1:pub mod cli;
core/crates/diagnostics/tests/cli_smoke.rs:1:use clap::Parser;
core/crates/diagnostics/examples/diagnostics_cli.rs:1:use clap::Parser;
```

## Tests executed
- cargo test --manifest-path core/crates/diagnostics/Cargo.toml
- (manual) cargo run -p craftcad_diagnostics --example diagnostics_cli --manifest-path core/Cargo.toml -- zip --preview

## Allowlist self-check
- Allowed paths only: YES
- No deletions: YES

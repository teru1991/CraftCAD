Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Usage:
#   powershell -ExecutionPolicy Bypass -File .\scripts\rust_quality.ps1

Set-Location (Join-Path $PSScriptRoot "..\ucel")

Write-Host "[rust-quality] rustfmt (check)"
cargo fmt --all -- --check

Write-Host "[rust-quality] cargo check (warnings as errors)"
$env:RUSTFLAGS = "-D warnings"
cargo check --workspace --all-targets
Remove-Item Env:\RUSTFLAGS -ErrorAction SilentlyContinue

Write-Host "[rust-quality] cargo test"
cargo test --workspace --all-targets

Write-Host "[rust-quality] cargo clippy (warnings as errors)"
cargo clippy --workspace --all-targets --all-features -- -D warnings

Write-Host "[rust-quality] cargo doc (warnings as errors)"
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --workspace --no-deps
Remove-Item Env:\RUSTDOCFLAGS -ErrorAction SilentlyContinue

Write-Host "[rust-quality] OK"

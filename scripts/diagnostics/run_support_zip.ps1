Param(
  [string]$Out = "diagnostics_out"
)
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..\..")
cargo run -p craftcad_diagnostics --example diagnostics_cli --manifest-path core/Cargo.toml -- zip --out $Out

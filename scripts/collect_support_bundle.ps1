param(
  [string]$BaseUrl = "http://127.0.0.1:8080",
  [string]$Root = "./artifacts/support_bundle"
)

$ts = Get-Date -Format "yyyyMMdd_HHmmss"
$outDir = Join-Path $Root $ts
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

Write-Host "[collect] base=$BaseUrl"
Write-Host "[collect] out=$outDir"

Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/healthz" -OutFile (Join-Path $outDir "healthz.json")
Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/support_bundle" -OutFile (Join-Path $outDir "support_bundle.json")
Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/metrics" -OutFile (Join-Path $outDir "metrics.prom")

@"
base_url=$BaseUrl
timestamp=$ts
files:
- healthz.json
- support_bundle.json
- metrics.prom
"@ | Out-File -Encoding utf8 (Join-Path $outDir "manifest.txt")

Write-Host "[ok] collected: $outDir"

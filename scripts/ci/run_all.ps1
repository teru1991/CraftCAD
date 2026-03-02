$ErrorActionPreference = 'Stop'

$RootDir = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$LogDir = Join-Path $RootDir '.ci_logs'
$SummaryScript = Join-Path $RootDir 'scripts/ci/parse_failures.py'
$SummaryFile = Join-Path $LogDir 'summary.json'

New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
Get-ChildItem -Path $LogDir -Filter '*.log' -ErrorAction SilentlyContinue | Remove-Item -Force
if (Test-Path $SummaryFile) { Remove-Item $SummaryFile -Force }

$script:OverallStatus = 0

function Invoke-Step {
    param(
        [string]$Name,
        [string]$WorkingDirectory,
        [scriptblock]$Command
    )

    $LogFile = Join-Path $LogDir ("$Name.log")
    "==> $Name" | Out-File -FilePath $LogFile -Encoding utf8

    Push-Location $WorkingDirectory
    try {
        & $Command *>> $LogFile
        if ($LASTEXITCODE -ne 0) { throw "Exit code $LASTEXITCODE" }
        "[PASS] $Name" | Out-File -FilePath $LogFile -Append -Encoding utf8
    }
    catch {
        "[FAIL] $Name" | Out-File -FilePath $LogFile -Append -Encoding utf8
        $script:OverallStatus = 1
    }
    finally {
        Pop-Location
    }
}

Invoke-Step -Name 'rust_fmt' -WorkingDirectory (Join-Path $RootDir 'core') -Command { cargo fmt --all -- --check }
Invoke-Step -Name 'rust_clippy' -WorkingDirectory (Join-Path $RootDir 'core') -Command { cargo clippy --workspace --all-targets -- -D warnings }
Invoke-Step -Name 'rust_test' -WorkingDirectory (Join-Path $RootDir 'core') -Command { cargo test --workspace --all-targets }

$DesktopCmake = Join-Path $RootDir 'apps/desktop/CMakeLists.txt'
if (Test-Path $DesktopCmake) {
    $DesktopBuildDir = Join-Path $RootDir 'build/desktop'

    Invoke-Step -Name 'rust_ffi_desktop' -WorkingDirectory (Join-Path $RootDir 'core') -Command { cargo build -p craftcad_ffi_desktop }

    Invoke-Step -Name 'rust_ffi_build' -WorkingDirectory (Join-Path $RootDir 'core') -Command { cargo build -p craftcad_ffi_desktop }
    Invoke-Step -Name 'cmake_configure' -WorkingDirectory $RootDir -Command { cmake -S apps/desktop -B $DesktopBuildDir -DCMAKE_BUILD_TYPE=Release }
    Invoke-Step -Name 'cmake_build' -WorkingDirectory $RootDir -Command { cmake --build $DesktopBuildDir --parallel }

    if ((Test-Path (Join-Path $DesktopBuildDir 'CTestTestfile.cmake')) -or (Test-Path (Join-Path $DesktopBuildDir 'Testing'))) {
        Invoke-Step -Name 'ctest' -WorkingDirectory $RootDir -Command { ctest --test-dir $DesktopBuildDir --output-on-failure }
    }
    else {
        "==> ctest`n[SKIP] ctest metadata not found in $DesktopBuildDir" | Out-File -FilePath (Join-Path $LogDir 'ctest.log') -Encoding utf8
    }
}

$pythonCmd = if (Get-Command python -ErrorAction SilentlyContinue) {
    'python'
} elseif (Get-Command py -ErrorAction SilentlyContinue) {
    'py -3'
} else {
    throw 'Python interpreter not found (python/py).'
}

Invoke-Expression "$pythonCmd \"$SummaryScript\" --log-dir \"$LogDir\" --out \"$SummaryFile\""
if ($LASTEXITCODE -ne 0) { exit 1 }

if ($script:OverallStatus -eq 0) {
    $summary = Get-Content $SummaryFile -Raw | ConvertFrom-Json
    if ($summary.total_failures -ne 0) {
        $script:OverallStatus = 1
    }
}

exit $script:OverallStatus

# Runs all .tong examples using the Rust tong CLI
param(
    [string]$TongExe
)

$ErrorActionPreference = 'Stop'

# Try to resolve tong executable
if (-not $TongExe) {
    $candidates = @(
        (Join-Path -Path $PSScriptRoot -ChildPath "..\rust\tong\target\release\tong.exe"),
        (Join-Path -Path $PSScriptRoot -ChildPath "..\rust\tong\target\debug\tong.exe"),
        (Join-Path -Path $env:USERPROFILE -ChildPath ".cargo\bin\tong.exe")
    )
    foreach ($c in $candidates) { if (Test-Path $c) { $TongExe = $c; break } }
}

if (-not $TongExe) {
    Write-Host "Building tong (debug)..."
    Push-Location (Join-Path $PSScriptRoot "..\rust\tong")
    try {
        cargo build | Out-Null
    }
    finally {
        Pop-Location | Out-Null
    }
    $TongExe = (Join-Path $PSScriptRoot "..\rust\tong\target\debug\tong.exe")
}

if (-not (Test-Path $TongExe)) { throw "Could not find or build tong.exe. Looked for: $TongExe" }

$examplesDir = (Join-Path $PSScriptRoot "..\examples")
$files = Get-ChildItem -Path $examplesDir -Filter *.tong -File | Sort-Object Name

foreach ($f in $files) {
    Write-Host "\n=== Running $($f.Name) ===" -ForegroundColor Cyan
    & $TongExe $f.FullName
}

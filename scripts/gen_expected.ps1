<#
Generate expected outputs for all non-SDL examples (PowerShell version).
Mirrors scripts/gen_expected.sh behavior.
Usage:
  pwsh scripts/gen_expected.ps1
Optional env vars:
  UPDATE (unused here; always (re)generates)
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $root

$outDir = Join-Path $root 'examples/expected'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Enumerate .tong files (excluding SDL module examples)
$examples = Get-ChildItem -Path (Join-Path $root 'examples') -Recurse -Filter *.tong | Where-Object { $_.FullName -notmatch "modules\\sdl\\" }

foreach ($f in $examples) {
  $rel = $f.FullName.Substring((Join-Path $root 'examples').Length) -replace '^[\\/]+',''
  # Remove .tong extension cleanly without leaving trailing dot
  $relNoExt = $rel -replace '\\.tong$',''
  $target = Join-Path $outDir ($relNoExt + '.out')
  $legacy = Join-Path $outDir ($relNoExt + '..out')
  if ((-not (Test-Path $target)) -and (Test-Path $legacy)) {
    # Migrate legacy double-dot file name
    Move-Item -Force -Path $legacy -Destination $target
  }
  $targetDir = Split-Path -Parent $target
  if (-not (Test-Path $targetDir)) { New-Item -ItemType Directory -Path $targetDir -Force | Out-Null }
  Write-Host "[gen] $rel -> examples/expected/$relNoExt.out"
  $cargoCmd = "cargo run --quiet --manifest-path rust/tong/Cargo.toml -- `"$($f.FullName)`""
  $output = Invoke-Expression $cargoCmd 2>&1
  Set-Content -Path $target -Value $output -Encoding UTF8
}

Write-Host "[gen] Done."

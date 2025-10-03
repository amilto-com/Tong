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

function Write-LFUtf8 {
  param([string]$Path, [string]$Text)
  $normalized = $Text -replace "`r?`n","`n"
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $normalized, $utf8NoBom)
}

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $root

$outDir = Join-Path $root 'examples/expected'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Enumerate .tong files (excluding SDL module examples)
$examples = Get-ChildItem -Path (Join-Path $root 'examples') -Recurse -Filter *.tong | Where-Object { $_.FullName -notmatch "modules\\sdl\\" }

foreach ($f in $examples) {
  $rel = $f.FullName.Substring((Join-Path $root 'examples').Length) -replace '^[\\/]+',''
  $baseName = [IO.Path]::GetFileNameWithoutExtension($rel)
  $target = Join-Path $outDir ($baseName + '.out')
  # Migrate legacy names if they exist (then normalize line endings)
  foreach ($legacy in @( (Join-Path $outDir ($baseName + '..out')), (Join-Path $outDir ($baseName + '.tong.out')) )) {
    if ((-not (Test-Path $target)) -and (Test-Path $legacy)) {
      Move-Item -Force -Path $legacy -Destination $target
      $migrated = Get-Content $target -Raw
      Write-LFUtf8 -Path $target -Text $migrated
    }
  }
  $targetDir = Split-Path -Parent $target
  if (-not (Test-Path $targetDir)) { New-Item -ItemType Directory -Path $targetDir -Force | Out-Null }
  Write-Host "[gen] $rel -> examples/expected/$baseName.out"
  $cargoCmd = "cargo run --quiet --manifest-path rust/tong/Cargo.toml -- `"$($f.FullName)`""
  $output = Invoke-Expression $cargoCmd 2>&1
  Write-LFUtf8 -Path $target -Text $output
}

# Final cleanup: remove any stray legacy files still present
Get-ChildItem -Path $outDir -File | Where-Object { $_.Name -match '\\.tong\.out$' -or $_.Name -match '\\.\.out$' } | ForEach-Object { Remove-Item $_.FullName -Force }

Write-Host "[gen] Done."

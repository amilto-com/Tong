<#
Comprehensive regression checker for all non-SDL examples (PowerShell).
Mirrors scripts/check_examples.sh.
Usage:
  pwsh scripts/check_examples.ps1
  pwsh scripts/check_examples.ps1 -Files hello.tong,math.tong
  UPDATE=1 pwsh scripts/check_examples.ps1  # update snapshots
Parameters:
  -Files   Comma or space separated list relative to examples/ or full path
Env:
  UPDATE=1   Update/create snapshots
#>
[CmdletBinding()]
param(
  [string]$Files
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $root

$update = ($env:UPDATE -eq '1')
$exampleRoot = Join-Path $root 'examples'
$expectedRoot = Join-Path $exampleRoot 'expected'

$allFiles = @()
if ($Files) {
    $split = $Files -split '[, ]+' | Where-Object { $_ }
    foreach ($item in $split) {
        $cand = $null
        if (Test-Path $item -PathType Leaf) { $cand = (Resolve-Path $item).Path }
        elseif (Test-Path (Join-Path $exampleRoot $item) -PathType Leaf) { $cand = (Resolve-Path (Join-Path $exampleRoot $item)).Path }
        if (-not $cand) { Write-Host "[SKIP] not found: $item"; continue }
        if ($cand -notmatch [regex]::Escape($exampleRoot)) { Write-Host "[SKIP] outside examples/: $cand"; continue }
        $allFiles += $cand
    }
} else {
    $allFiles = Get-ChildItem -Path $exampleRoot -Recurse -Filter *.tong | ForEach-Object { $_.FullName }
}

$fail = $false
$total = 0; $pass = 0; $updated = 0

foreach ($f in $allFiles) {
    if ($f -match 'modules\\sdl\\') { continue }
    $rel = ($f.Substring($exampleRoot.Length) -replace '^[\\/]+' , '')
    $relNoExt = $rel -replace '\\.tong$',''
    $expected = Join-Path $expectedRoot ($relNoExt + '.out')
    $legacy = Join-Path $expectedRoot ($relNoExt + '..out')
    if (-not (Test-Path $expected) -and (Test-Path $legacy)) {
        Move-Item -Force -Path $legacy -Destination $expected
    }
    $total++
    Write-Host "[RUN ] $rel"
    $tmp = New-TemporaryFile
    try {
        $cargoCmd = "cargo run --quiet --manifest-path rust/tong/Cargo.toml -- `"$f`""
        $output = Invoke-Expression $cargoCmd 2>&1
        Set-Content -Path $tmp -Value $output -Encoding UTF8
    } catch {
        Write-Host "[FAIL] runtime error: $rel" -ForegroundColor Red
        Get-Content $tmp | ForEach-Object { '    ' + $_ }
        $fail = $true; Remove-Item $tmp -Force; continue
    }
    if (-not (Test-Path $expected)) {
        if ($update) {
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $expected) | Out-Null
            Copy-Item $tmp $expected
            Write-Host "[CREATE] $rel (snapshot)"
            $pass++; $updated++
        } else {
            Write-Host "[MISS] expected missing: $expected" -ForegroundColor Yellow
            $fail = $true
        }
        Remove-Item $tmp -Force; continue
    }
    $expectedContent = Get-Content $expected -Raw -ErrorAction SilentlyContinue
    $actualContent = Get-Content $tmp -Raw -ErrorAction SilentlyContinue
    if ($expectedContent -eq $actualContent) {
        Write-Host "[PASS] $rel"
        $pass++
    } else {
        if ($update) {
            Copy-Item $tmp $expected -Force
            Write-Host "[UPDATE] $rel"
            $pass++; $updated++
        } else {
            Write-Host "[DIFF] $rel" -ForegroundColor Red
            # Simple diff (line by line)
            $expLines = ($expectedContent -split '\r?\n')
            $actLines = ($actualContent -split '\r?\n')
            $max = [Math]::Max($expLines.Length, $actLines.Length)
            for ($i=0; $i -lt $max; $i++) {
                $a = if ($i -lt $expLines.Length) { $expLines[$i] } else { '' }
                $b = if ($i -lt $actLines.Length) { $actLines[$i] } else { '' }
                if ($a -ne $b) { Write-Host ("- $a`n+ $b") }
            }
            $fail = $true
        }
    }
    Remove-Item $tmp -Force
}

Write-Host "== Summary =="
Write-Host "Total: $total  Passed: $pass  Failed: $(($total - $pass))  Updated: $updated"
if ($fail) {
    if ($updated -gt 0) { Write-Host "[RESULT] MIXED (some updated, some failed)" -ForegroundColor Yellow } else { Write-Host "[RESULT] FAIL" -ForegroundColor Red }
    exit 1
} else {
    if ($updated -gt 0) { Write-Host "[RESULT] OK (updated $updated snapshots)" -ForegroundColor Green } else { Write-Host "[RESULT] OK" -ForegroundColor Green }
    exit 0
}

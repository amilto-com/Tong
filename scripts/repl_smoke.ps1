<#
REPL smoke test (PowerShell version) mirroring scripts/repl_smoke.sh
Usage:
  pwsh scripts/repl_smoke.ps1            # compare against snapshot
  UPDATE=1 pwsh scripts/repl_smoke.ps1   # (re)generate snapshot
#>
[CmdletBinding()]
param(
        [switch]$Update
)
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
$snapshot = Join-Path $root 'examples/expected/repl_smoke.out'
$tmp = New-TemporaryFile

# Feed content (keep in sync with repl_smoke.sh)
$feed = @'
let x = 10
let y = 32
print("sum", x + y)

# data + pattern functions
data Maybe = Nothing | Just v
def fromMaybe(Just(v)) { v }
def fromMaybe(Nothing) { 0 }
print("fmJ", fromMaybe(Just(5)))
print("fmN", fromMaybe(Nothing))

# guarded factorial
def fact(0) { 1 }
def fact(n) if n > 0 { n * fact(n - 1) }
print("fact5", fact(5))

# lambdas (backslash + pipe) and partials
let inc = \a -> a + 1
let sq = |n| n * n
print("inc41", inc(41))
print("sq7", sq(7))
let add = \a b -> a + b
print("add2_3", add(2,3))

# list comprehensions (single + multi) + predicate
let nums = [1,2,3,4,5]
print("squares", [ x*x | x in nums ])
print("pairs", [ (x,y) | x in nums, y in nums if x < y & x + y < 7 ])

# logical operators & || ! and short-circuit demo
let side = [0]
let short1 = false & (side[0] = side[0] + 1)
let tmpS = side[0]
print("short1", short1, tmpS)
let short2 = true || (side[0] = side[0] + 1)
print("short2", short2, side[0])
let short3 = !false & true || false
print("logicMix", short3)

# array element update sugar
let arr = [0,1,2]
arr[1] = arr[1] + 10
print("arr", arr[0], arr[1], arr[2])

# anonymous fn with block and nested indexing
let make = fn a b {
    let c = a + b
    let d = c * 2
    d + 1
}
print("make", make(2,3))
let grid = [[1,2],[3,4]]
print("grid10", grid[1][0])

# match with guard
match Just(42) { Just(v) if v > 40 -> print("matchJ", v), Nothing -> print("matchN") }

:quit
'@

# Execute REPL via cargo run piping feed
try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = 'cargo'
    $psi.ArgumentList.Add('run')
    $psi.ArgumentList.Add('--quiet')
    $psi.ArgumentList.Add('--manifest-path')
    $psi.ArgumentList.Add('rust/tong/Cargo.toml')
    $psi.ArgumentList.Add('--')
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $proc = New-Object System.Diagnostics.Process
    $proc.StartInfo = $psi
    [void]$proc.Start()
    foreach ($line in ($feed -split "`r?`n")) { $proc.StandardInput.WriteLine($line) }
    $proc.StandardInput.Close()
    $stdOut = $proc.StandardOutput.ReadToEnd()
    $stdErr = $proc.StandardError.ReadToEnd()
    $proc.WaitForExit()
    $combined = ($stdOut + $stdErr)
    Write-LFUtf8 -Path $tmp -Text $combined
    if ($proc.ExitCode -ne 0) { throw "REPL exited with code $($proc.ExitCode)" }
}
catch {
    Write-Host '[REPL] execution failed' -ForegroundColor Red
    if (Test-Path $tmp) { Get-Content $tmp | ForEach-Object { '  | ' + $_ } }
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    exit 1
}

if ($Update) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $snapshot) | Out-Null
    $tmpContent = Get-Content $tmp -Raw
    Write-LFUtf8 -Path $snapshot -Text $tmpContent
    Write-Host "[repl] snapshot (re)generated at $snapshot"
    Remove-Item $tmp -Force
    exit 0
}

if (-not (Test-Path $snapshot)) {
    Write-Host "[repl] missing snapshot: $snapshot (run with UPDATE=1)" -ForegroundColor Yellow
    Remove-Item $tmp -Force
    exit 1
}

$exp = Get-Content $snapshot -Raw
$act = Get-Content $tmp -Raw
if ($exp -eq $act) {
    Write-Host '[repl] PASS'
    Remove-Item $tmp -Force
    exit 0
} else {
    Write-Host '[repl] DIFF' -ForegroundColor Red
    $expL = $exp -split '\r?\n'
    $actL = $act -split '\r?\n'
    $max = [Math]::Max($expL.Length, $actL.Length)
    for ($i=0; $i -lt $max; $i++) {
        $a = if ($i -lt $expL.Length) { $expL[$i] } else { '' }
        $b = if ($i -lt $actL.Length) { $actL[$i] } else { '' }
        if ($a -ne $b) { Write-Host ("- $a`n+ $b") }
    }
    Remove-Item $tmp -Force
    exit 1
}

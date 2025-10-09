<#
.SYNOPSIS
    HTTP server for Tong WASM REPL
.PARAMETER Port
    Port to listen on (default: 8080)
#>
[CmdletBinding()]
param([int]$Port = 8080)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$webDir = Join-Path $root 'web'

if (-not (Test-Path $webDir)) {
    Write-Host "Error: web directory not found at $webDir" -ForegroundColor Red
    Write-Host "Run 'cargo build --target wasm32-unknown-unknown --release --no-default-features --features wasm' first" -ForegroundColor Yellow
    exit 1
}

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")

try {
    $listener.Start()
    Write-Host "🌐 Tong WASM REPL at http://localhost:$Port/" -ForegroundColor Green
    Write-Host "   Root: $webDir" -ForegroundColor Cyan
    Write-Host "   Press Ctrl+C to stop`n" -ForegroundColor Yellow

    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response
        
        $urlPath = $request.Url.LocalPath.TrimStart('/') -replace '\\','/'
        if ($urlPath -eq '') { $urlPath = 'index.html' }
        
        $filePath = Join-Path $webDir $urlPath
        $filePath = [System.IO.Path]::GetFullPath($filePath)
        
        # Security: prevent directory traversal
        if (-not $filePath.StartsWith($webDir)) {
            $response.StatusCode = 403
            $response.Close()
            continue
        }

        Write-Host "$($request.HttpMethod) /$urlPath" -ForegroundColor Gray

        if (Test-Path $filePath -PathType Leaf) {
            # Serve file with correct MIME type
            $ext = [System.IO.Path]::GetExtension($filePath).ToLower()
            $contentType = switch ($ext) {
                '.wasm' { 'application/wasm' }
                '.html' { 'text/html; charset=utf-8' }
                '.js'   { 'application/javascript; charset=utf-8' }
                '.json' { 'application/json; charset=utf-8' }
                '.css'  { 'text/css; charset=utf-8' }
                default { 'application/octet-stream' }
            }
            
            $response.ContentType = $contentType
            # Required headers for SharedArrayBuffer (if needed in future)
            $response.AddHeader('Cross-Origin-Opener-Policy', 'same-origin')
            $response.AddHeader('Cross-Origin-Embedder-Policy', 'require-corp')
            
            $fileStream = [System.IO.File]::OpenRead($filePath)
            $response.ContentLength64 = $fileStream.Length
            $fileStream.CopyTo($response.OutputStream)
            $fileStream.Close()
            
        } else {
            # 404
            $response.StatusCode = 404
            $html = "<html><body><h1>404 Not Found</h1><p>/$urlPath</p></body></html>"
            $buffer = [System.Text.Encoding]::UTF8.GetBytes($html)
            $response.ContentType = 'text/html; charset=utf-8'
            $response.ContentLength64 = $buffer.Length
            $response.OutputStream.Write($buffer, 0, $buffer.Length)
        }
        
        $response.Close()
    }
}
finally {
    if ($listener -and $listener.IsListening) {
        $listener.Stop()
    }
    if ($listener) {
        $listener.Close()
    }
    Write-Host "`nServer stopped." -ForegroundColor Yellow
}

# T00 tracer — build the wasm proof page, embed it in the hub, build the hub (release).
# Run from spikes\tracer:   powershell -ExecutionPolicy Bypass -File build.ps1
# Needs: dx 0.7.x (`dx --version`) and the wasm target (`rustup target add wasm32-unknown-unknown`).
$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
Set-Location $here

Write-Host "==> 1/3 dx build (wasm-board, web, release)" -ForegroundColor Cyan
Push-Location "$here\wasm-board"
dx build --platform web --release
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    Write-Host "dx build failed. The hub still works without the wasm page: run 'cargo run --release -p tracer-hub'." -ForegroundColor Yellow
    Write-Host "Note the error for gym-network.md (it is T00 finding #2: the wasm pipeline)." -ForegroundColor Yellow
    exit 1
}
Pop-Location

Write-Host "==> 2/3 copy dx output into hub\assets\wasm" -ForegroundColor Cyan
# dx 0.7 writes to target\dx\<app>\release\web\public; search so a layout change doesn't break the script.
$index = Get-ChildItem -Path "$here\target\dx\wasm-board" -Recurse -Filter index.html -ErrorAction SilentlyContinue |
    Where-Object { $_.DirectoryName -like "*public*" } | Select-Object -First 1
if (-not $index) { throw "dx output not found under target\dx\wasm-board (look for public\index.html)" }
$public = $index.DirectoryName
Write-Host "    from $public"
Remove-Item -Recurse -Force "$here\hub\assets\wasm\*"
Copy-Item -Recurse -Force "$public\*" "$here\hub\assets\wasm\"

Write-Host "==> 3/3 cargo build --release -p tracer-hub (assets embedded)" -ForegroundColor Cyan
cargo build --release -p tracer-hub
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$exe = "$here\target\release\tracer-hub.exe"
$size = [math]::Round((Get-Item $exe).Length / 1MB, 1)
Write-Host ""
Write-Host "Built $exe ($size MB) - copy this ONE file to the gym laptop; it needs nothing else." -ForegroundColor Green
Write-Host "Run it:  .\target\release\tracer-hub.exe       (window with QR)"
Write-Host "    or:  .\target\release\tracer-hub.exe --headless"

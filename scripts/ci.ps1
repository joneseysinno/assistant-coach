# Assistant Coach CI (Windows / PowerShell). Run from the repo root:
#   powershell -ExecutionPolicy Bypass -File scripts\ci.ps1
# Same steps as scripts/ci.sh and .github/workflows/ci.yml.
$ErrorActionPreference = "Stop"

function Step($name, [scriptblock]$cmd) {
    Write-Host "==> $name" -ForegroundColor Cyan
    & $cmd
    if ($LASTEXITCODE -ne 0) { Write-Host "FAILED: $name" -ForegroundColor Red; exit $LASTEXITCODE }
}

Step "boundaries (review-checklist)" { python scripts/check_boundaries.py }
Step "fixtures oracle (p0_check)"    { python tools/p0_check.py docs/fixtures/games }
Step "cargo fmt --check"             { cargo fmt --all -- --check }
Step "cargo clippy -D warnings"      { cargo clippy --workspace --all-targets -- -D warnings }
Step "cargo test"                    { cargo test --workspace }
Write-Host "CI green." -ForegroundColor Green

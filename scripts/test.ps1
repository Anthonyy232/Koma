# Run app.test.contract against a development build, headless, with Exact's driver.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot

Push-Location $root
try {
    cargo build -p koma-windows
    if ($LASTEXITCODE) { throw 'build failed' }
} finally {
    Pop-Location
}

$env:EXACT_APP_DIR = $root
# Names the binary, so the driver doesn't build one of its own.
$env:EXACT_WINDOWS_BIN = Join-Path $root 'target\debug\Koma.exe'
Push-Location (Join-Path $root 'vendor\exact')
try {
    bun scripts/agent.mjs windows --test (Join-Path $root 'app.test.contract')
    exit $LASTEXITCODE
} finally {
    Pop-Location
}

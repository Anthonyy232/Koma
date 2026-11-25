# Everything a change must pass: formatting, lints, unit tests, then the contract tests.
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot

Push-Location $root
try {
    cargo fmt --check
    if ($LASTEXITCODE) { throw 'cargo fmt found unformatted code' }
    cargo clippy --workspace --all-targets -- -D warnings
    if ($LASTEXITCODE) { throw 'clippy failed' }
    cargo test --workspace
    if ($LASTEXITCODE) { throw 'unit tests failed' }
} finally {
    Pop-Location
}
& (Join-Path $PSScriptRoot 'test.ps1')
exit $LASTEXITCODE

# Fetch the Exact checkout Koma builds against, at the pinned commit, with Koma's patch applied,
# and the Exact packages Koma uses. Needs git, rustup and Bun 1.4.2 (the version Exact pins).
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot
$exact = Join-Path $root 'vendor\exact'
$commit = (Get-Content (Join-Path $root 'patches\exact.commit')).Trim()

# Windows PowerShell carries on past a failing native command.
function Invoke-Native {
    & $args[0] @($args | Select-Object -Skip 1)
    if ($LASTEXITCODE) { throw "failed: $args" }
}

if (-not (Test-Path $exact)) {
    Invoke-Native git init --quiet $exact
    Invoke-Native git -C $exact remote add origin https://github.com/expo/exact.git
}
Invoke-Native git -C $exact fetch --depth 1 origin $commit
Invoke-Native git -C $exact checkout --quiet --force FETCH_HEAD
Invoke-Native git -C $exact apply --whitespace=nowarn (Join-Path $root 'patches\exact-overlay.patch')

# Exact's build scripts run its own tooling, which resolves offline.
Push-Location $exact
try {
    Invoke-Native bun install --frozen-lockfile
    Invoke-Native cargo fetch --locked
} finally {
    Pop-Location
}
Push-Location $root
try {
    Invoke-Native bun install --frozen-lockfile
    # Koma's build asks Cargo about its own crates offline too.
    Invoke-Native cargo fetch --locked
} finally {
    Pop-Location
}

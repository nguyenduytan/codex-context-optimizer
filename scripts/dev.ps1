[CmdletBinding()]
param([ValidateSet('test','check','build','fmt')][string]$Action = 'check')
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$localCargo = Join-Path $root '.tools/cargo/bin/cargo.exe'
if (Test-Path -LiteralPath $localCargo) {
    $env:RUSTUP_HOME = Join-Path $root '.tools/rustup'
    $env:CARGO_HOME = Join-Path $root '.tools/cargo'
    $cargo = $localCargo
} else { $cargo = (Get-Command cargo -ErrorAction Stop).Source }
Push-Location $root
try {
    if ($Action -eq 'fmt') { & $cargo fmt --all; if ($LASTEXITCODE) { throw 'Formatting failed' }; return }
    if ($Action -eq 'check') {
        & $cargo fmt --all --check; if ($LASTEXITCODE) { throw 'Formatting check failed' }
        & $cargo clippy --workspace --all-targets --all-features -- -D warnings; if ($LASTEXITCODE) { throw 'Clippy failed' }
    }
    & $cargo build --workspace --all-features --locked; if ($LASTEXITCODE) { throw 'Build failed' }
    if ($Action -ne 'build') { & $cargo test --workspace --locked; if ($LASTEXITCODE) { throw 'Tests failed' } }
} finally { Pop-Location }

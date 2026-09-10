[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9-]{0,38}$')]
    [string]$Owner,
    [ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_.-]+$')]
    [string]$Name = 'codex-context-optimizer'
)
$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$old = 'YOUR_GITHUB_USER/codex-context-optimizer'
$new = "$Owner/$Name"
$files = @('Cargo.toml', 'README.md', 'docs/installation.md', 'docs/publishing.md', 'SECURITY.md', 'CONTRIBUTING.md', 'scripts/install.ps1', 'scripts/install.sh', '.github/ISSUE_TEMPLATE/config.yml')
$utf8 = New-Object Text.UTF8Encoding($false)
foreach ($relative in $files) {
    $path = Join-Path $root $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing expected file: $relative" }
    $text = [IO.File]::ReadAllText($path)
    $updated = $text.Replace($old, $new)
    if ($text -ne $updated -and $PSCmdlet.ShouldProcess($relative, "Replace repository placeholder with $new")) {
        [IO.File]::WriteAllText($path, $updated, $utf8)
    }
}
Write-Host "Repository links prepared for $new. Review git diff; no Git remote or external repository was changed."

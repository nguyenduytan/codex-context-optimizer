[CmdletBinding()]
param(
    [ValidatePattern('^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$')]
    [string]$Repository = 'nguyenduytan/codex-context-optimizer',
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$')]
    [string]$Version = 'v0.1.0',
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'ctxc/bin'),
    [switch]$Force
)
$ErrorActionPreference = 'Stop'
if (-not [IO.Path]::IsPathRooted($InstallDir)) { throw 'InstallDir must be absolute.' }
$architecture = $env:PROCESSOR_ARCHITEW6432
if (-not $architecture) { $architecture = $env:PROCESSOR_ARCHITECTURE }
switch ($architecture) {
    'AMD64' { $target = 'x86_64-pc-windows-msvc' }
    'ARM64' { $target = 'aarch64-pc-windows-msvc' }
    default { throw "Unsupported architecture: $architecture" }
}
$resolvedDir = [IO.Path]::GetFullPath($InstallDir)
$cursor = $resolvedDir
while ($cursor) {
    if ((Test-Path -LiteralPath $cursor) -and
        ((Get-Item -LiteralPath $cursor -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw 'Refusing reparse-point installation path.'
    }
    $cursor = [IO.Path]::GetDirectoryName($cursor)
}
$destination = Join-Path $resolvedDir 'ctxc.exe'
if (Test-Path -LiteralPath $destination) {
    if ((Get-Item -LiteralPath $destination -Force).Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw 'Refusing linked binary target.'
    }
    if (-not $Force) { throw 'ctxc.exe exists. Use -Force to explicitly replace it.' }
}
$asset = "ctxc-$Version-$target.zip"
$url = "https://github.com/$Repository/releases/download/$Version"
$tempDir = Join-Path ([IO.Path]::GetTempPath()) ('ctxc-install-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $tempDir
$stage = $null
try {
    $archive = Join-Path $tempDir 'archive.zip'
    $checksums = Join-Path $tempDir 'SHA256SUMS'
    Invoke-WebRequest "$url/$asset" -OutFile $archive -UseBasicParsing
    Invoke-WebRequest "$url/SHA256SUMS" -OutFile $checksums -UseBasicParsing
    $matchesFound = @(Get-Content -LiteralPath $checksums | Where-Object {
        $_ -match ('^[a-fA-F0-9]{64}\s+' + [regex]::Escape($asset) + '$')
    })
    if ($matchesFound.Count -ne 1) { throw 'Missing or ambiguous checksum entry.' }
    $expected = ($matchesFound[0] -split '\s+')[0]
    if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne $expected) {
        throw 'Checksum mismatch; nothing installed.'
    }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($archive)
    try {
        $entries = @($zip.Entries | Where-Object { $_.FullName -eq 'ctxc.exe' })
        if ($entries.Count -ne 1 -or $entries[0].Length -eq 0 -or $entries[0].Length -gt 200MB) {
            throw 'Archive must contain one nonempty root ctxc.exe of at most 200 MiB.'
        }
        [IO.Compression.ZipFileExtensions]::ExtractToFile(
            $entries[0], (Join-Path $tempDir 'ctxc.exe'), $false)
    } finally { $zip.Dispose() }
    $null = New-Item -ItemType Directory -Force -Path $resolvedDir
    $stage = Join-Path $resolvedDir ('.ctxc-install-' + [Guid]::NewGuid().ToString('N') + '.exe')
    Copy-Item -LiteralPath (Join-Path $tempDir 'ctxc.exe') -Destination $stage
    if (Test-Path -LiteralPath $destination) { [IO.File]::Replace($stage, $destination, $null) }
    else { [IO.File]::Move($stage, $destination) }
    Write-Host "Installed $Version to $destination"
    Write-Host "Add $resolvedDir to PATH if needed. No PATH or Codex config was modified."
} finally {
    foreach ($name in @('archive.zip', 'SHA256SUMS', 'ctxc.exe')) {
        $temporaryFile = Join-Path $tempDir $name
        if (Test-Path -LiteralPath $temporaryFile) { Remove-Item -LiteralPath $temporaryFile -Force }
    }
    if ($stage -and (Test-Path -LiteralPath $stage)) { Remove-Item -LiteralPath $stage -Force }
    Remove-Item -LiteralPath $tempDir
}

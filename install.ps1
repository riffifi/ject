param(
    [ValidateSet('install', 'update', 'uninstall')]
    [string]$Command = 'install',
    [ValidatePattern('^(latest|v[0-9][a-zA-Z0-9.+-]*)$')]
    [string]$Version = 'latest',
    [string]$BinDir = $env:JECT_BIN_DIR,
    [switch]$NoPath
)

$ErrorActionPreference = 'Stop'
if (!$BinDir) {
    $BinDir = if ($MyInvocation.MyCommand.Name -eq 'jectup.ps1') {
        $PSScriptRoot
    } else {
        Join-Path $env:LOCALAPPDATA 'Ject\bin'
    }
}
$BinDir = [IO.Path]::GetFullPath($BinDir)
$executable = Join-Path $BinDir 'ject.exe'
$manager = Join-Path $BinDir 'jectup.ps1'
$receipt = Join-Path $BinDir '.ject-managed'
$record = if (Test-Path -LiteralPath $receipt -PathType Leaf) {
    (Get-Content -LiteralPath $receipt -Raw).Trim()
} else { '' }
$ownsInstall = $record -in @('ject-release-installer', 'ject-release-installer:path')
$ownsPath = $record -eq 'ject-release-installer:path'

function Update-Path([bool]$Add) {
    if ($NoPath -or !$ownsPath) { return }
    $previous = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @($previous -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $BinDir.TrimEnd('\') })
    if ($Add) { $parts = @($BinDir) + $parts }
    [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User')

    $sessionParts = @($env:Path -split ';' | Where-Object { $_ -and $_.TrimEnd('\') -ne $BinDir.TrimEnd('\') })
    if ($Add) { $sessionParts = @($BinDir) + $sessionParts }
    $env:Path = $sessionParts -join ';'
}

if ($Command -eq 'uninstall') {
    if (!$ownsInstall) { throw "No installer-managed Ject in $BinDir" }
    foreach ($file in @($executable, $manager, $receipt)) {
        if (Test-Path -LiteralPath $file -PathType Leaf) { Remove-Item -LiteralPath $file -Force }
    }
    Update-Path $false
    Write-Output "Uninstalled Ject from $BinDir"
    return
}

if (!$ownsInstall) {
    if ($Command -eq 'update') { throw "No installer-managed Ject in $BinDir. Run install first." }
    foreach ($file in @($executable, $manager, $receipt)) {
        if (Test-Path -LiteralPath $file) { throw "$file belongs to another installation" }
    }
}
$alreadyOnPath = @([Environment]::GetEnvironmentVariable('Path', 'User') -split ';' |
    Where-Object { $_ -and $_.TrimEnd('\') -eq $BinDir.TrimEnd('\') }).Count -gt 0
if (!$NoPath -and !$alreadyOnPath) { $ownsPath = $true }

if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'Prebuilt Windows releases currently require x64 Windows.'
}
$asset = 'ject-x86_64-pc-windows-msvc.exe'
$release = if ($Version -eq 'latest') {
    'https://github.com/riffifi/ject/releases/latest/download'
} else {
    "https://github.com/riffifi/ject/releases/download/$Version"
}
$null = New-Item -ItemType Directory -Path $BinDir -Force
$scratch = Join-Path $BinDir ".install-$([guid]::NewGuid())"
$null = New-Item -ItemType Directory -Path $scratch

try {
    foreach ($name in @($asset, "$asset.sha256")) {
        $destination = Join-Path $scratch $name
        if ($env:JECT_RELEASE_DIR) {
            Copy-Item -LiteralPath (Join-Path $env:JECT_RELEASE_DIR $name) -Destination $destination
        } else {
            Invoke-WebRequest -UseBasicParsing -Uri "$release/$name" -OutFile $destination
        }
    }
    $staged = Join-Path $scratch $asset
    $expected = ((Get-Content -LiteralPath "$staged.sha256" -Raw).Trim() -split '\s+')[0]
    if ($expected -notmatch '^[0-9a-f]{64}$' -or (Get-FileHash -Algorithm SHA256 -LiteralPath $staged).Hash -ne $expected) {
        throw "Checksum mismatch for $asset"
    }
    $reported = & $staged --version
    if ($LASTEXITCODE -ne 0 -or $reported -notmatch '^ject [0-9]') { throw 'Downloaded Ject could not start' }
    if ($Version -ne 'latest' -and $reported -ne "ject $($Version.Substring(1))") {
        throw "Wanted $Version, downloaded $reported"
    }
    if (Test-Path -LiteralPath $executable -PathType Leaf) {
        [IO.File]::Replace($staged, $executable, (Join-Path $scratch 'previous.exe'))
    } else {
        [IO.File]::Move($staged, $executable)
    }
    if ($PSCommandPath -ne $manager) {
        Copy-Item -LiteralPath $PSCommandPath -Destination $manager -Force
    }
    $record = if ($ownsPath) { 'ject-release-installer:path' } else { 'ject-release-installer' }
    Set-Content -LiteralPath $receipt -Value $record -Encoding ascii
    Update-Path $true
    Write-Output "Installed $reported at $executable"
} finally {
    Remove-Item -LiteralPath $scratch -Recurse -Force
}

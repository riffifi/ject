param([Parameter(Mandatory)][string]$Binary)
$ErrorActionPreference = 'Stop'
$source = (Resolve-Path $Binary).Path
$installer = Join-Path $PSScriptRoot '../install.ps1'
$scratch = Join-Path ([IO.Path]::GetTempPath()) "ject-install-$([guid]::NewGuid())"
$null = New-Item -ItemType Directory -Path $scratch
$originalPath = $env:Path
$originalUserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$pathOptions = if ($IsWindows) { @{} } else { @{NoPath = $true} }

try {
    $bin = Join-Path $scratch 'my bin'
    $env:JECT_RELEASE_DIR = Join-Path $scratch 'assets'
    $null = New-Item -ItemType Directory -Path $env:JECT_RELEASE_DIR
    $asset = Join-Path $env:JECT_RELEASE_DIR (Split-Path $source -Leaf)
    Copy-Item -LiteralPath $source -Destination $asset
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $asset).Hash
    Set-Content -LiteralPath "$asset.sha256" -Value $hash
    $expected = & $source --version
    $tag = $expected -replace '^ject ', 'v'
    & $installer install $tag -BinDir $bin @pathOptions
    $executable = Join-Path $bin 'ject.exe'
    $manager = Join-Path $bin 'jectup.ps1'
    if ((& $executable --version) -ne $expected) { throw 'Installed version differs' }
    if ($IsWindows -and (Get-Command ject -CommandType Application).Source -ne $executable) {
        throw 'Installed executable is missing from PATH'
    }

    Set-Content -LiteralPath "$asset.sha256" -Value ('0' * 64)
    $failed = $false
    try { & $manager update @pathOptions } catch { $failed = $_.Exception.Message -like 'Checksum mismatch*' }
    if (!$failed) { throw 'Expected the checksum failure' }
    if ((& $executable --version) -ne $expected) { throw 'Failed update changed the executable' }
    Set-Content -LiteralPath "$asset.sha256" -Value $hash
    & $manager update @pathOptions
    if ((& $executable --version) -ne $expected) { throw 'Updated version differs' }
    & $manager uninstall @pathOptions
    if ((Test-Path -LiteralPath $executable) -or (Test-Path -LiteralPath $manager)) {
        throw 'Uninstall left Ject files'
    }
    if ($IsWindows -and $bin -in ([Environment]::GetEnvironmentVariable('Path', 'User') -split ';')) {
        throw 'Uninstall left the managed PATH entry'
    }
    Set-Content -LiteralPath $executable -Value 'foreign executable'
    $failed = $false
    try { & $installer install -BinDir $bin -NoPath } catch {
        $failed = $_.Exception.Message -like '*belongs to another installation'
    }
    if (!$failed -or (Get-Content -LiteralPath $executable) -ne 'foreign executable') {
        throw 'Install must preserve unmanaged files'
    }
} finally {
    if ($IsWindows) {
        [Environment]::SetEnvironmentVariable('Path', $originalUserPath, 'User')
        $env:Path = $originalPath
    }
    Remove-Item -LiteralPath $scratch -Recurse -Force
}

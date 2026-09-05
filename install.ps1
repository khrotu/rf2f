<#
.SYNOPSIS
    Installs rf2f.
.DESCRIPTION
    Downloads the executable, registers it, and sets up the PATH.
.EXAMPLE
    irm https://raw.githubusercontent.com/khrotu/rf2f/main/install.ps1 | iex
.EXAMPLE
    .\install.ps1 -InstallDir "$env:LOCALAPPDATA\rf2f" -Force
.NOTES
    To uninstall, run `rf2f unregister`, then delete the install directory.
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'rf2f'),
    [string]$ExeUrl = 'https://raw.githubusercontent.com/khrotu/rf2f/main/target/release/rf2f.exe',
    [string]$IconUrl = 'https://raw.githubusercontent.com/khrotu/rf2f/main/assets/logo.ico',
    [switch]$Force
)
$ErrorActionPreference = 'Stop'
$exePath = Join-Path $InstallDir 'rf2f.exe'
$iconPath = Join-Path $InstallDir 'logo.ico'
if ((Test-Path $exePath) -and (-not $Force)) {
    throw "rf2f is already installed at $exePath. Re-run with -Force to overwrite, or run `rf2f unregister` first."
}
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Write-Host "Downloading rf2f.exe ..."
Invoke-WebRequest -Uri $ExeUrl -OutFile $exePath -UseBasicParsing
try {
    Write-Host "Downloading logo.ico ..."
    Invoke-WebRequest -Uri $IconUrl -OutFile $iconPath -UseBasicParsing
} catch {
    Write-Warning "Icon download failed ($($_.Exception.Message))."
}
Write-Host "Registering ..."
& $exePath register
if ($LASTEXITCODE -ne 0) {
    throw "Register failed with exit code $LASTEXITCODE."
}
Write-Host ""
Write-Host "Done! Installed to $exePath"

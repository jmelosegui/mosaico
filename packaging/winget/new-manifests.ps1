<#
.SYNOPSIS
Writes the winget manifests for a release's MSI and validates them.

.DESCRIPTION
The first version of a package in microsoft/winget-pkgs is submitted by
hand; the release workflow's winget job then opens a pull request for each
later version, starting from the latest one there. This script writes the
three manifests of that first submission (version, installer and en-US
locale) under target\winget\manifests\j\jmelosegui\mosaico\<version>,
the layout winget-pkgs uses, and runs `winget validate` on them.

The installer hash and ProductCode come from the MSI itself: the one
published with the release, downloaded, or a local one given with -MsiPath
for a dry run. Copy the folder into a fork of winget-pkgs at the same path
to submit it, or pass it to `wingetcreate submit`.

.EXAMPLE
pwsh packaging\winget\new-manifests.ps1 -Version 0.13.0

.EXAMPLE
pwsh packaging\winget\new-manifests.ps1 -Version 0.13.0 -MsiPath target\release\mosaico-windows-amd64.msi
#>
param(
    [Parameter(Mandatory)] [string] $Version,
    [string] $MsiPath,
    [string] $OutDir = (Join-Path $PSScriptRoot "..\..\target\winget")
)

$ErrorActionPreference = "Stop"

$Identifier = "jmelosegui.mosaico"
$ManifestVersion = "1.12.0"
$Tag = "v$Version"
$Url = "https://github.com/jmelosegui/mosaico/releases/download/$Tag/mosaico-windows-amd64.msi"

if (-not $MsiPath) {
    $MsiPath = Join-Path ([IO.Path]::GetTempPath()) "mosaico-$Version.msi"
    Write-Host "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $MsiPath
}
$MsiPath = (Resolve-Path $MsiPath).Path

# Reads a value of the MSI's Property table.
function Get-MsiProperty {
    param($Database, $Name)
    $View = $Database.OpenView("SELECT ``Value`` FROM ``Property`` WHERE ``Property`` = '$Name'")
    [void]$View.Execute()
    $Record = $View.Fetch()
    [void]$View.Close()
    if (-not $Record) { throw "The MSI has no $Name" }
    $Record.StringData(1)
}

$Installer = New-Object -ComObject WindowsInstaller.Installer
$Database = $Installer.OpenDatabase($MsiPath, 0)
$ProductCode = Get-MsiProperty $Database "ProductCode"
$UpgradeCode = Get-MsiProperty $Database "UpgradeCode"
$ProductVersion = Get-MsiProperty $Database "ProductVersion"
$ProductName = Get-MsiProperty $Database "ProductName"
$Manufacturer = Get-MsiProperty $Database "Manufacturer"
[void][Runtime.InteropServices.Marshal]::ReleaseComObject($Database)

if ($ProductVersion -ne $Version) {
    throw "The MSI is version $ProductVersion, not $Version"
}
$Sha256 = (Get-FileHash $MsiPath -Algorithm SHA256).Hash

$Dir = Join-Path $OutDir "manifests\j\jmelosegui\mosaico\$Version"
New-Item -ItemType Directory -Force -Path $Dir | Out-Null

$Header = "# yaml-language-server: `$schema=https://aka.ms/winget-manifest"

@"
$Header.version.$ManifestVersion.schema.json

PackageIdentifier: $Identifier
PackageVersion: $Version
DefaultLocale: en-US
ManifestType: version
ManifestVersion: $ManifestVersion
"@ | Set-Content (Join-Path $Dir "$Identifier.yaml") -Encoding utf8NoBOM

@"
$Header.installer.$ManifestVersion.schema.json

PackageIdentifier: $Identifier
PackageVersion: $Version
Platform:
- Windows.Desktop
InstallerType: msi
Scope: user
InstallModes:
- interactive
- silent
- silentWithProgress
UpgradeBehavior: install
Commands:
- mosaico
AppsAndFeaturesEntries:
- DisplayName: $ProductName
  Publisher: $Manufacturer
  ProductCode: '$ProductCode'
  UpgradeCode: '$UpgradeCode'
Installers:
- Architecture: x64
  InstallerUrl: $Url
  InstallerSha256: $Sha256
  ProductCode: '$ProductCode'
ManifestType: installer
ManifestVersion: $ManifestVersion
"@ | Set-Content (Join-Path $Dir "$Identifier.installer.yaml") -Encoding utf8NoBOM

@"
$Header.defaultLocale.$ManifestVersion.schema.json

PackageIdentifier: $Identifier
PackageVersion: $Version
PackageLocale: en-US
Publisher: $Manufacturer
PublisherUrl: https://github.com/jmelosegui
PublisherSupportUrl: https://github.com/jmelosegui/mosaico/issues
Author: Juan Manuel Elosegui
PackageName: $ProductName
PackageUrl: https://jmelosegui.com/mosaico/
License: MIT
LicenseUrl: https://github.com/jmelosegui/mosaico/blob/main/LICENSE
ShortDescription: A fast, configurable tiling window manager for Windows, written in Rust.
Description: |-
  Mosaico tiles your windows automatically with BSP, vertical stack and three column layouts, with up to 8
  workspaces per monitor, vim-style keybindings, colored borders on the focused window, a status bar, per-app
  rules and hot-reloaded configuration. It installs for the current user and adds the mosaico command and a
  Start menu entry for its settings app.
Moniker: mosaico
Tags:
- tiling
- tiling-window-manager
- window-manager
- productivity
- workspaces
- rust
ReleaseNotesUrl: https://github.com/jmelosegui/mosaico/releases/tag/$Tag
Documentations:
- DocumentLabel: Documentation
  DocumentUrl: https://jmelosegui.com/mosaico/
ManifestType: defaultLocale
ManifestVersion: $ManifestVersion
"@ | Set-Content (Join-Path $Dir "$Identifier.locale.en-US.yaml") -Encoding utf8NoBOM

Write-Host "Wrote the manifests to $Dir"
winget validate --manifest $Dir
if ($LASTEXITCODE -ne 0) { throw "winget validate failed with code $LASTEXITCODE" }

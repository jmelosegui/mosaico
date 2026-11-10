# Mosaico installer script for Windows
# Usage: irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex
#
# Installs the latest release's MSI (mosaico-windows-amd64.msi) for the
# current user, into %LOCALAPPDATA%\Programs\mosaico. Releases without an
# MSI fall back to the portable zip (mosaico-windows-amd64.zip), which is
# copied into %LOCALAPPDATA%\mosaico as before.
#
# A copy installed from the zip by an earlier version of this script is
# moved to the MSI: its daemon is stopped, its executables and PATH entry
# are removed, and its autostart and running daemon are carried over to
# the installed copy.

$ErrorActionPreference = "Stop"

$Repo = "jmelosegui/mosaico"
$LegacyDir = "$env:LOCALAPPDATA\mosaico"
$MsiName = "mosaico-windows-amd64.msi"
$ZipName = "mosaico-windows-amd64.zip"

function Write-Info { param($msg) Write-Host "==> " -ForegroundColor Green -NoNewline; Write-Host $msg }
# Throws rather than exits: under `iex`, exit would close the user's shell.
function Write-Err { param($msg) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $msg; throw $msg }

# Ends the mosaico processes, daemon and settings app, that run from $Dir.
function Stop-MosaicoIn {
    param($Dir)
    Get-Process mosaico, mosaico-settings -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and ((Split-Path $_.Path) -eq $Dir.TrimEnd("\")) } |
        Stop-Process -Force
}

# Runs mosaico and shows its output. Through the pipe PowerShell decodes
# that output with the console's code page, which garbles mosaico's UTF-8
# (the banner's logo) unless it is UTF-8 for the call. The user's code
# page is put back after: under `iex` this is their session.
function Show-Mosaico {
    param($Exe, [string[]]$Arguments)
    $Previous = [Console]::OutputEncoding
    try {
        [Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
        & $Exe @Arguments | Out-Host
    } finally {
        [Console]::OutputEncoding = $Previous
    }
}

function Remove-FromUserPath {
    param($Dir)
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $UserPath) { return }
    $Kept = $UserPath.Split(";") | Where-Object { $_ -and ($_.TrimEnd("\") -ne $Dir.TrimEnd("\")) }
    $NewPath = $Kept -join ";"
    if ($NewPath -ne $UserPath) {
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    }
}

function Install-Msi {
    param($Url)
    $MsiPath = Join-Path $TempDir $MsiName
    if (Test-Path $Url -PathType Leaf) {
        Copy-Item $Url $MsiPath
    } else {
        Write-Info "Downloading $Url..."
        Invoke-WebRequest -Uri $Url -OutFile $MsiPath
    }

    # Carry a copy installed from the zip over to the MSI: note whether
    # its daemon runs and whether autostart starts it, then stop it.
    $Legacy = Join-Path $LegacyDir "mosaico.exe"
    $LegacyRunning = $false
    $LegacyAutostart = $false
    if (Test-Path $Legacy) {
        Write-Info "Moving the copy in $LegacyDir to the installer..."
        $LegacyRunning = [bool](Get-Process mosaico -ErrorAction SilentlyContinue |
            Where-Object { $_.Path -eq $Legacy })
        $Status = (& $Legacy autostart status 2>$null) -join "`n"
        $LegacyAutostart = $Status.Trim() -eq "Autostart is currently enabled."
        # `stop` puts back the windows the daemon hid; ending it would not.
        if ($LegacyRunning) { & $Legacy stop | Out-Null }
        Stop-MosaicoIn $LegacyDir
    }

    Write-Info "Running the installer..."
    $Log = Join-Path $TempBase "mosaico-install.log"
    $Arguments = "/i `"$MsiPath`" /passive /norestart /l*v `"$Log`""
    $Process = Start-Process msiexec.exe -ArgumentList $Arguments -Wait -PassThru
    # 3010: installed, a restart is required to finish.
    if ($Process.ExitCode -notin 0, 3010) {
        if ($LegacyRunning) { Show-Mosaico $Legacy start }
        Write-Err "The installer failed with code $($Process.ExitCode). Its log is $Log"
    }

    $InstallDir = (Get-ItemProperty "HKCU:\Software\mosaico" -ErrorAction SilentlyContinue).InstallFolder
    if (-not $InstallDir) { $InstallDir = "$env:LOCALAPPDATA\Programs\mosaico\" }
    $Mosaico = Join-Path $InstallDir "mosaico.exe"

    if (Test-Path $Legacy) {
        # Only the executables: the folder also holds mosaico's data.
        Get-ChildItem $LegacyDir -File |
            Where-Object { $_.Name -match '^mosaico(-settings)?\.exe(\.old)?$' } |
            Remove-Item -Force -ErrorAction SilentlyContinue
        Remove-FromUserPath $LegacyDir
        if ($LegacyAutostart) { & $Mosaico autostart enable | Out-Null }
        if ($LegacyRunning) { Show-Mosaico $Mosaico start }
    }

    # This session's PATH: the installer only updated the user's, which
    # new sessions read.
    $env:Path = ($env:Path.Split(";") | Where-Object { $_ -and ($_.TrimEnd("\") -ne $LegacyDir) }) -join ";"
    if ($env:Path -notlike "*$($InstallDir.TrimEnd('\'))*") { $env:Path = "$env:Path;$InstallDir" }
    return $Mosaico
}

function Install-Zip {
    param($Url)
    $ZipPath = Join-Path $TempDir $ZipName
    Write-Info "Downloading $Url..."
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath

    Write-Info "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    # Stop the daemon and the settings app before replacing their binaries
    Stop-MosaicoIn $LegacyDir

    Write-Info "Installing to $LegacyDir..."
    New-Item -ItemType Directory -Force -Path $LegacyDir | Out-Null
    foreach ($Exe in Get-ChildItem -Path $TempDir -Filter *.exe) {
        $Target = Join-Path $LegacyDir $Exe.Name
        Copy-Item -Path $Exe.FullName -Destination $Target -Force
        Unblock-File -Path $Target
    }

    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$LegacyDir*") {
        Write-Info "Adding $LegacyDir to PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$LegacyDir", "User")
        $env:Path = "$env:Path;$LegacyDir"
    }
    return Join-Path $LegacyDir "mosaico.exe"
}

Write-Info "Installing mosaico..."

$TempBase = (Get-Item $env:TEMP).FullName
$TempDir = Join-Path $TempBase "mosaico-install-$PID"

# MOSAICO_INSTALL_MSI names an MSI on disk to install instead of the
# latest release's, which is how CI tests this script.
if ($env:MOSAICO_INSTALL_MSI) {
    $Package = (Resolve-Path $env:MOSAICO_INSTALL_MSI).Path
    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
    $Mosaico = Install-Msi $Package
} else {
    try {
        $Headers = @{ "User-Agent" = "mosaico-installer" }
        $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers $Headers
        $Version = $Release.tag_name
        Write-Info "Latest version: $Version"
    } catch {
        Write-Err "Could not determine latest version. Check https://github.com/$Repo/releases"
    }

    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
    $Download = "https://github.com/$Repo/releases/download/$Version"
    if ($Release.assets | Where-Object { $_.name -eq $MsiName }) {
        $Mosaico = Install-Msi "$Download/$MsiName"
    } else {
        $Mosaico = Install-Zip "$Download/$ZipName"
    }
}

# Cleanup
Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue

# Verify
Write-Info "Successfully installed mosaico!"
& $Mosaico --version

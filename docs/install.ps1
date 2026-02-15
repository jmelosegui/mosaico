# Mosaico installer script for Windows
# Usage: irm https://mosaico.dev/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "jmelosegui/mosaico"
$InstallDir = "$env:LOCALAPPDATA\mosaico"

function Write-Info { param($msg) Write-Host "==> " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn { param($msg) Write-Host "warning: " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err { param($msg) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $msg; exit 1 }

Write-Info "Installing mosaico..."

# Get latest release version
try {
    $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $Release.tag_name
    Write-Info "Latest version: $Version"
} catch {
    Write-Err "Could not determine latest version. Check https://github.com/$Repo/releases"
}

# Download
$Filename = "mosaico-windows-amd64.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Filename"
$TempBase = (Get-Item $env:TEMP).FullName
$TempDir = Join-Path $TempBase "mosaico-install-$PID"
$ZipPath = Join-Path $TempDir $Filename

Write-Info "Downloading $Url..."

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
Invoke-WebRequest -Uri $Url -OutFile $ZipPath

# Extract
Write-Info "Extracting..."
Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

# Install
Write-Info "Installing to $InstallDir..."
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Move-Item -Path (Join-Path $TempDir "mosaico.exe") -Destination (Join-Path $InstallDir "mosaico.exe") -Force

# Cleanup
Remove-Item -Recurse -Force $TempDir

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# Refresh PATH in current session
$env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [Environment]::GetEnvironmentVariable("Path", "User")

# Reload PowerShell profile if it exists
if (Test-Path $PROFILE) {
    Write-Info "Reloading PowerShell profile..."
    . $PROFILE
}

# Verify
Write-Info "Successfully installed mosaico!"
& "$InstallDir\mosaico.exe" --version

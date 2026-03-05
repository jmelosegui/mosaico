param(
    [Parameter(Mandatory=$false)]
    [string]$OutputPath,

    [Parameter(Mandatory=$false)]
    [ValidateSet("primary", "all")]
    [string]$Mode = "primary"
)

# Default output path when none is provided.
if (-not $OutputPath) {
    $OutputPath = Join-Path $env:TEMP "mosaico_screenshot.png"
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# Ensure output directory exists.
$dir = Split-Path -Parent $OutputPath
if ($dir -and -not (Test-Path $dir)) {
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}

if ($Mode -eq "all") {
    # Capture the virtual screen (all monitors).
    $bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
} else {
    # Capture the primary monitor only.
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
}

$bitmap  = New-Object System.Drawing.Bitmap($bounds.Width, $bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bitmap.Save($OutputPath)
$graphics.Dispose()
$bitmap.Dispose()

Write-Output $OutputPath

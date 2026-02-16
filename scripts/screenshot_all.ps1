Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$vScreen = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bitmap = New-Object System.Drawing.Bitmap($vScreen.Width, $vScreen.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen($vScreen.Left, $vScreen.Top, 0, 0, $vScreen.Size)
$bitmap.Save("C:\Users\jmelo\Pictures\Screenshots\both_monitors.png")
$graphics.Dispose()
$bitmap.Dispose()

Write-Output "Captured $($vScreen.Width)x$($vScreen.Height) from ($($vScreen.Left),$($vScreen.Top))"

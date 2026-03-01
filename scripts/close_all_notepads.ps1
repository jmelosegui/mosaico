Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;

public class Win32Close {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int RealGetWindowClass(IntPtr hWnd, StringBuilder pszType, int cchType);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool PostMessageW(IntPtr hWnd, uint msg, UIntPtr wParam, IntPtr lParam);

    public static List<IntPtr> AllWindows = new List<IntPtr>();
}
"@

$WM_CLOSE = 0x0010
$count = 0

[Win32Close]::AllWindows.Clear()
[Win32Close]::EnumWindows(
    [Win32Close+EnumWindowsProc]{ param($h, $l); [Win32Close]::AllWindows.Add($h); return $true },
    [IntPtr]::Zero
) | Out-Null

foreach ($h in [Win32Close]::AllWindows) {
    $classBuf = New-Object Text.StringBuilder 256
    [Win32Close]::RealGetWindowClass($h, $classBuf, 256) | Out-Null
    $cls = $classBuf.ToString()

    if ($cls -eq "Notepad") {
        [Win32Close]::PostMessageW($h, $WM_CLOSE, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
        $count++
    }
}

if ($count -gt 0) {
    Write-Host "Sent WM_CLOSE to $count Notepad window(s). Waiting..."
    Start-Sleep -Seconds 3
} else {
    Write-Host "No Notepad windows found."
}

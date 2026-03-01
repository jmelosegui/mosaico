Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;

public class Win32Check {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern int GetWindowTextLength(IntPtr hWnd);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int RealGetWindowClass(IntPtr hWnd, StringBuilder pszType, int cchType);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern int GetWindowLong(IntPtr hWnd, int nIndex);

    [DllImport("user32.dll")]
    public static extern IntPtr GetWindow(IntPtr hWnd, uint uCmd);

    [DllImport("dwmapi.dll")]
    public static extern int DwmGetWindowAttribute(IntPtr hWnd, int dwAttribute, out int pvAttribute, int cbAttribute);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left, Top, Right, Bottom;
    }

    public static List<IntPtr> AllWindows = new List<IntPtr>();

    public static bool Callback(IntPtr hWnd, IntPtr lParam) {
        AllWindows.Add(hWnd);
        return true;
    }
}
"@

[Win32Check]::AllWindows.Clear()
[Win32Check]::EnumWindows([Win32Check+EnumWindowsProc]{ param($h, $l); [Win32Check]::AllWindows.Add($h); return $true }, [IntPtr]::Zero) | Out-Null

$GWL_STYLE = -16
$GWL_EXSTYLE = -20
$GW_OWNER = 4
$WS_CAPTION = 0x00C00000
$WS_EX_TOOLWINDOW = 0x00000080
$WS_EX_DLGMODALFRAME = 0x00000001
$WS_EX_NOREDIRECTIONBITMAP = 0x00200000
$WS_EX_APPWINDOW = 0x00040000

foreach ($h in [Win32Check]::AllWindows) {
    $classBuf = New-Object Text.StringBuilder 256
    [Win32Check]::RealGetWindowClass($h, $classBuf, 256) | Out-Null
    $cls = $classBuf.ToString()

    if ($cls -ne "Notepad") { continue }

    $visible = [Win32Check]::IsWindowVisible($h)
    $titleBuf = New-Object Text.StringBuilder 256
    [Win32Check]::GetWindowText($h, $titleBuf, 256) | Out-Null
    $title = $titleBuf.ToString()

    $style = [Win32Check]::GetWindowLong($h, $GWL_STYLE)
    $exstyle = [Win32Check]::GetWindowLong($h, $GWL_EXSTYLE)

    $hasCaption = ($style -band $WS_CAPTION) -eq $WS_CAPTION
    $isTool = ($exstyle -band $WS_EX_TOOLWINDOW) -ne 0
    $isDialog = ($exstyle -band $WS_EX_DLGMODALFRAME) -ne 0
    $noRedir = ($exstyle -band $WS_EX_NOREDIRECTIONBITMAP) -ne 0
    $appWindow = ($exstyle -band $WS_EX_APPWINDOW) -ne 0

    $owner = [Win32Check]::GetWindow($h, $GW_OWNER)
    $hasOwner = $owner -ne [IntPtr]::Zero

    $cloaked = 0
    [Win32Check]::DwmGetWindowAttribute($h, 14, [ref]$cloaked, 4) | Out-Null

    $rect = New-Object Win32Check+RECT
    [Win32Check]::GetWindowRect($h, [ref]$rect) | Out-Null
    $w = $rect.Right - $rect.Left
    $ht = $rect.Bottom - $rect.Top

    Write-Host ("0x{0:X8} vis={1} caption={2} tool={3} dialog={4} owner={5} noRedir={6} appWnd={7} cloaked={8} size={9}x{10} title=[{11}]" -f `
        $h.ToInt64(), $visible, $hasCaption, $isTool, $isDialog, $hasOwner, $noRedir, $appWindow, $cloaked, $w, $ht, $title)
}

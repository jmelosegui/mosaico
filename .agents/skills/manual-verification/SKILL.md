---
name: manual-verification
description: Verify mosaico behaviour on the live desktop before reporting a feature as working. Use when a change affects tiling, borders, the bar, workspaces, monitors or the daemon.
---

# Manual verification

Test it yourself before asking the user to verify anything. You have the CLI, IPC commands and PowerShell.

## Loop

1. Kill the daemon: `taskkill /IM mosaico.exe /F`.
2. Rebuild and start it.
3. Exercise the feature with `mosaico.exe action ...`.
4. Check observable state programmatically: `mosaico debug list` for managed windows, PowerShell with
   `DwmGetWindowAttribute` for cloak state, `IsWindowVisible` for visibility.
5. Read the daemon log at `~/.config/mosaico/logs/mosaico.log` for errors or unexpected behaviour.
6. Take screenshots to check layout, borders, monocle and workspace switching.
7. Report the result only after it is confirmed.

## Screenshots

`scripts/screenshot.ps1` captures the screen and prints the output path. Read the image afterwards.

```powershell
# Primary monitor to $env:TEMP/mosaico_screenshot.png
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/screenshot.ps1

# Primary monitor to a specific path
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/screenshot.ps1 -OutputPath path/to/output.png

# All monitors
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/screenshot.ps1 -OutputPath path/to/output.png -Mode all
```

- `-OutputPath` (optional): where to save the PNG. Parent directories are created.
- `-Mode` (optional): `primary` (default) or `all` for the whole virtual screen.

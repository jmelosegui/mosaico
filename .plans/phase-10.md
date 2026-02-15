# Phase 10: Multi-Monitor Support

**Status:** Complete

**Goal:** Detect all connected monitors at startup, assign windows to the
correct monitor's workspace, and tile each monitor independently. Add actions
to move focus and windows between monitors.

## Design

- Each monitor gets its own `Workspace` and cached work area.
- `TilingManager` manages a sorted list of `MonitorState` (left-to-right).
- Window events route to the correct monitor via `MonitorFromWindow`.
- Focus/swap actions operate within the focused monitor's workspace.
- New actions allow cross-monitor focus switching and window movement.

## New actions

| Action              | Default Keybinding  | Description                          |
|---------------------|---------------------|--------------------------------------|
| FocusMonitorNext    | Alt + Shift + L     | Focus a window on the next monitor   |
| FocusMonitorPrev    | Alt + Shift + H     | Focus a window on the previous monitor |
| MoveToMonitorNext   | Alt + Ctrl + L      | Move focused window to next monitor  |
| MoveToMonitorPrev   | Alt + Ctrl + H      | Move focused window to previous monitor |

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      action.rs             # Add 4 new action variants
      config.rs             # Add default keybindings for monitor actions
  mosaico-windows/
    src/
      monitor.rs            # Add MonitorInfo, enumerate_monitors(), monitor_id_for_window()
      window.rs             # Add set_foreground() method
      tiling.rs             # Rewrite for per-monitor workspaces
  mosaico/
    src/
      main.rs               # Add CLI subcommands for monitor actions
```

## Tasks

- [x] Add `FocusMonitorNext/Prev` and `MoveToMonitorNext/Prev` to Action enum
- [x] Add default keybindings for monitor actions in config
- [x] Add `MonitorInfo` struct and `enumerate_monitors()` to monitor.rs
- [x] Add `monitor_id_for_window()` helper
- [x] Move `set_foreground` to Window struct
- [x] Rewrite TilingManager with per-monitor workspaces
- [x] Add CLI subcommands for the new actions
- [x] Build, lint, test
- [x] Commit

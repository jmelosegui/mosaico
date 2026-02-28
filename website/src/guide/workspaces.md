# Workspaces

Mosaico supports up to 8 independent workspaces per monitor. Each workspace
maintains its own set of tiled windows.

## Switching Workspaces

| Shortcut | Action |
|----------|--------|
| `Alt + 1` -- `Alt + 8` | Switch to workspace 1-8 |
| `Alt + Shift + 1` -- `Alt + Shift + 8` | Send focused window to workspace 1-8 |

Or via the CLI:

```sh
mosaico action goto-workspace 3
mosaico action send-to-workspace 5
```

## Behavior

### Switching (`goto-workspace`)

When you switch to a different workspace:

1. All windows on the current workspace are hidden
2. All windows on the target workspace are shown
3. The layout is reapplied
4. Focus is set to the first window

If you are already on the target workspace, nothing happens.

### Sending (`send-to-workspace`)

When you send a window to another workspace:

1. The focused window is removed from the current workspace
2. It is added to the target workspace
3. The window is hidden (it is now on a non-active workspace)
4. The current workspace is retiled
5. Focus moves to the next window

## Per-Monitor Workspaces

Each monitor has its own independent set of 8 workspaces. Switching
workspaces on one monitor does not affect other monitors.

## Status Bar Integration

If the status bar is enabled with a `workspaces` widget, it displays
pill-shaped indicators showing which workspace is active and which
workspaces contain windows.

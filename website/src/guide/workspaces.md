# Workspaces

Mosaico supports up to 8 workspaces. Each workspace maintains its own set
of tiled windows. By default each monitor switches workspaces independently;
you can opt into a global mode where every monitor switches in lockstep
(see [Workspace Mode](#workspace-mode) below).

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

1. All windows on the current workspace are hidden using the configured
   hiding strategy
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

## Hiding Behaviour

The `hiding` setting in `[layout]` controls how windows are hidden during
workspace switches:

| Strategy | Taskbar Icon | Notes |
|----------|-------------|-------|
| `"cloak"` (default) | Kept | Invisible via DWM. Same mechanism as Windows virtual desktops. |
| `"hide"` | Removed | Legacy `SW_HIDE`. Simple but loses taskbar icons. |
| `"minimize"` | Kept (minimized) | Some apps react to minimize (e.g. pause media). |

```toml
[layout]
hiding = "cloak"   # "cloak", "hide", or "minimize"
```

**Cloak** is the recommended default. Windows become invisible but keep
their taskbar icons, so you can still see all running apps. Clicking a
cloaked window's taskbar icon automatically switches to its workspace.

## Workspace Mode

`workspaces.mode` in `config.toml` controls how a workspace switch
propagates across monitors:

```toml
[workspaces]
mode = "per-monitor"   # default
# mode = "global"
```

| Mode | Behavior |
|------|----------|
| `"per-monitor"` (default) | Switching workspaces only affects the focused monitor. The other monitors keep showing whatever workspace they were on. |
| `"global"` | Every monitor switches to the same workspace number in lockstep, mirroring Windows virtual desktops. |

In both modes each monitor still owns its own window list per workspace;
windows do not jump between displays. The mode only controls which
monitors flip when you press `Alt + N` or run `mosaico action goto-workspace N`.

`send-to-workspace` in `global` mode moves the window to the target
workspace on its current monitor, then flips every other monitor to the
same workspace number so the view stays in sync.

The mode is hot-reloaded: change it in `config.toml` and the next
workspace switch follows the new mode without restarting the daemon.

## Status Bar Integration

If the status bar is enabled with a `workspaces` widget, it displays
pill-shaped indicators showing which workspace is active and which
workspaces contain windows.

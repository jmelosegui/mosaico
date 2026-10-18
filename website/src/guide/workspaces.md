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
mosaico action go-to-workspace 3
mosaico action go-to-workspace-next
mosaico action go-to-workspace-prev
mosaico action send-to-workspace 5
```

`goto-workspace-next` and `goto-workspace-prev` step to the workspace after
or before the one on screen. They have no default shortcut; bind them in
`keybindings.toml` (see [Keybindings](keybindings.md)). A step past the last
workspace stays there, unless `wrap` is on:

```toml
[workspaces]
wrap = true   # past the last workspace, go back to the first
```

You can also click a workspace number on the [status bar](status-bar.md#the-mouse),
or scroll over the bar, to switch the workspaces of that bar's monitor.

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

When you send a window to another workspace, Mosaico goes there with it:

1. The focused window is moved from the current workspace to the target
   workspace
2. The other windows on the current workspace are hidden
3. The target workspace becomes the active one, and its windows are shown
4. The layout is reapplied
5. Focus stays on the window you sent

In global mode, the other monitors switch to the target workspace too. If
you are already on the target workspace, nothing happens.

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
monitors flip when you press `Alt + 1` to `Alt + 8` or run `mosaico action go-to-workspace N`.

`send-to-workspace` in `global` mode moves the window to the target
workspace on its current monitor, then flips every other monitor to the
same workspace number so the view stays in sync.

The mode is hot-reloaded: change it in `config.toml` and the next
workspace switch follows the new mode without restarting the daemon. So is
`wrap`.

A click on a workspace number of the status bar follows the mode too: in
global mode every monitor switches, and focus goes to the monitor whose bar
you clicked.

## Status Bar Integration

If the status bar is enabled with a `workspaces` widget, it displays
pill-shaped indicators showing which workspace is active and which
workspaces contain windows. Click one to go to that workspace on that
monitor, or scroll over the bar to step through them.

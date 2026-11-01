# Cheatsheet

A one-page reference for the default Mosaico keyboard shortcuts and CLI
actions. Print it, pin it, or just keep this tab open.

> **Tip:** every shortcut shown here can be remapped in
> `~/.config/mosaico/keybindings.toml`. See
> [Keyboard Bindings](keybindings.md) for the full configuration
> reference.

## At a Glance

```
Alt + H/J/K/L            focus left/down/up/right
Alt + Shift + H/J/K/L    move/swap window left/down/up/right
Alt + 1 .. Alt + 8       switch to workspace N
Alt + Shift + 1 .. 8     send focused window to workspace N
Alt + N                  cycle layout (BSP -> VStack -> 3Col)
Alt + T                  toggle monocle (full-screen one window)
Alt + F                  float the focused window, or tile it again
Alt + Shift + F          focus the next floating window
Alt + . / Alt + ,        widen / narrow the focused floating window
Alt + Shift + . / ,      heighten / shorten the focused floating window
Alt + M                  minimize focused window
Alt + Q                  close focused window
Alt + Shift + R          retile (re-apply current layout, splits back)
Alt + Shift + P          pause / unpause all mosaico hotkeys
```

## Focus

Move keyboard focus between tiled windows. Left and right cross monitors
when no neighbor exists in that direction; up and down stay on the
current monitor.

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + H` | Focus left | `mosaico action focus left` |
| `Alt + J` | Focus down | `mosaico action focus down` |
| `Alt + K` | Focus up | `mosaico action focus up` |
| `Alt + L` | Focus right | `mosaico action focus right` |

## Move

Swap the focused window with its neighbor in the given direction. Left
and right can move a window between adjacent monitors; up and down swap
within the same monitor.

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + Shift + H` | Move left | `mosaico action move left` |
| `Alt + Shift + J` | Move down | `mosaico action move down` |
| `Alt + Shift + K` | Move up | `mosaico action move up` |
| `Alt + Shift + L` | Move right | `mosaico action move right` |

## Workspaces

Each monitor has up to 8 workspaces. The default mode is *per-monitor*:
switching workspace only affects the focused monitor. Set
`workspaces.mode = "global"` in `config.toml` to flip every monitor in
lockstep, like Windows virtual desktops. See
[Workspaces](workspaces.md).

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + 1` | Switch to workspace 1 | `mosaico action go-to-workspace 1` |
| `Alt + 2` | Switch to workspace 2 | `mosaico action go-to-workspace 2` |
| ... | ... | ... |
| `Alt + 8` | Switch to workspace 8 | `mosaico action go-to-workspace 8` |
| `Alt + Shift + 1` | Send window to workspace 1 | `mosaico action send-to-workspace 1` |
| `Alt + Shift + 2` | Send window to workspace 2 | `mosaico action send-to-workspace 2` |
| ... | ... | ... |
| `Alt + Shift + 8` | Send window to workspace 8 | `mosaico action send-to-workspace 8` |
| -- | Next workspace | `mosaico action go-to-workspace-next` |
| -- | Previous workspace | `mosaico action go-to-workspace-prev` |

On the [status bar](status-bar.md#the-mouse), click a workspace number to go
there on that monitor, or scroll over the bar to step through the
workspaces.

## Layout

Cycle through Mosaico's three tiling algorithms or switch the focused
workspace into monocle mode (one window fills the work area). Retile
forces a fresh re-application of the current layout, with every split you
moved back at the configured ratio, or an equal share in a stack.

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + N` | Cycle layout (BSP -> VerticalStack -> ThreeColumn -> ...) | `mosaico action cycle-layout` |
| `Alt + T` | Toggle monocle mode | `mosaico action toggle-monocle` |
| `Alt + Shift + R` | Retile | `mosaico action retile` |

On the [status bar](status-bar.md#the-mouse), click the layout widget to
cycle the layout of that monitor, or right-click it to toggle monocle there.

Per-workspace layout overrides are configured under
`[workspaces.layouts]` in `config.toml`. See
[Tiling & Layouts](tiling-layout.md).

## Window

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + Q` | Close focused window | `mosaico action close-focused` |
| `Alt + M` | Minimize focused window | -- |

## Mouse

Drag a tiled window by its title bar; a preview shows where it will land.
Set `drag_to_swap = false` under `[mouse]` in `config.toml` to turn this
off. See [Dragging Windows](tiling-layout.md#dragging-windows).

| Drop it | Result |
|---------|--------|
| On another tile | The two swap places |
| On another monitor | It joins there, with its new slot under the pointer |
| On its own slot, in a gap, or with `Esc` pressed | It goes back to its slot |

Drag the edge between two tiles to resize them, or a corner where two
splits meet to move both; `Alt + Shift + R` puts the splits back. Set
`resize_splits = false` under `[mouse]` in `config.toml` to turn this off.
See [Resizing Tiles](tiling-layout.md#resizing-tiles).

## Daemon

Pause every Mosaico hotkey (including the pause hotkey itself stays
active so you can resume) when running a full-screen application that
needs to claim its own keybindings. The status bar shows a red `PAUSED`
pill while paused; click it to resume.

| Shortcut | Action | CLI |
|----------|--------|-----|
| `Alt + Shift + P` | Toggle pause / resume | `mosaico pause` / `mosaico unpause` |
| -- | Start the daemon | `mosaico start` |
| -- | Stop the daemon | `mosaico stop` |
| -- | Check status | `mosaico status` |
| -- | Reload config (automatic) | edit `config.toml`, save |
| -- | Diagnose setup | `mosaico doctor` |
| -- | Open the settings window | `mosaico settings`, or click the tray icon |

The tray icon menu also pauses and resumes hotkeys, reloads the config, opens
the config folder or the log, and quits mosaico. See
[Settings Window](settings.md#tray-icon).

## Modifier Keys

| Keyword | Key |
|---------|-----|
| `alt` | Alt |
| `shift` | Shift |
| `ctrl` | Control |
| `win` | Windows / Super |

Combine in `keybindings.toml` with
`modifiers = ["alt", "shift"]`.

## Recognised Key Names

| Type | Examples |
|------|----------|
| Letters | `A` .. `Z` (case-insensitive) |
| Digits | `0` .. `9` |
| Function | `F1` .. `F24` |
| Named | `Enter`, `Escape`, `Tab`, `Space`, `Backspace`, `Delete`, `Insert`, `Home`, `End`, `PageUp`, `PageDown`, `Up`, `Down`, `Left`, `Right` |

Set them in `~/.config/mosaico/keybindings.toml` like this:

```toml
[[keybinding]]
action = "focus-left"
key = "H"
modifiers = ["alt"]

[[keybinding]]
action = "send-to-workspace-3"
key = "3"
modifiers = ["alt", "shift"]
```

## Themes at a Glance

```toml
[theme]
name = "catppuccin"   # catppuccin, rose-pine, tokyo-night
flavor = "mocha"      # depends on family (see below)
```

| Family | Flavors |
|--------|---------|
| `catppuccin` | `mocha`, `macchiato`, `frappe`, `latte` |
| `rose-pine` | `main`, `moon`, `dawn` |
| `tokyo-night` | `night`, `storm`, `day` |

See [Theming](theming.md) for palette colors per family.

## Border Colors at a Glance

```toml
[borders]
width = 4              # 0 disables every border
corner_style = "small" # square / small / round

[borders.colors]
focused = ""           # empty = theme default
monocle = ""           # empty = theme default
unfocused = ""         # empty = theme default; "none" disables unfocused borders
```

See [Focus Borders](focus-border.md) for the full lifecycle.

## See Also

- [Configuration](configuration.md) -- every config field, all four files.
- [Keyboard Bindings](keybindings.md) -- full action reference and how to
  remap.
- [CLI Commands](cli.md) -- start, stop, status, action, doctor, ...

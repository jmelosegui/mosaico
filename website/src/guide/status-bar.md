# Status Bar

Mosaico includes a configurable status bar that sits at the top of each
monitor. It is configured in `~/.config/mosaico/bar.toml`.

## Configuration

```toml
enabled = true
height = 28
monitor = "all"      # "all", "primary", or 0-based index

[colors]
background = "base"      # Named theme color or hex
foreground = "text"
accent = "blue"

[[left]]
type = "workspaces"

[[center]]
type = "clock"
format = "%H:%M"

[[right]]
type = "cpu"
```

## General Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `enabled` | bool | `true` | Show or hide the bar |
| `height` | int | `28` | Bar height in pixels |
| `monitor` | string | `"all"` | Which monitors show the bar |

The `monitor` field accepts:
- `"all"` -- show on every monitor
- `"primary"` -- show only on the primary monitor
- A number (e.g. `0`, `1`) -- show on a specific monitor by index

## Colors

Colors can be specified as hex values (`"#1e1e2e"`) or as named theme
colors (`"blue"`, `"green"`, `"mauve"`, etc.). The named-color palette
depends on the active theme family (Catppuccin, Rosé Pine, Tokyo Night);
see [Theming](theming.md) for the full list per family.

| Setting | Description |
|---------|-------------|
| `background` | Bar background color |
| `foreground` | Default text color |
| `accent` | Highlight color for active elements |

## Widgets

Widgets are placed in `[[left]]`, `[[center]]`, or `[[right]]` arrays.

| Widget | Description | Options |
|--------|-------------|---------|
| `workspaces` | Workspace indicator pills | -- |
| `active_window` | Icon of the focused window | -- |
| `layout` | Current layout mode | -- |
| `clock` | Current time | `format` (strftime) |
| `date` | Current date | `format` (strftime) |
| `ram` | RAM usage percentage | -- |
| `cpu` | CPU usage percentage | -- |
| `update` | Update indicator | -- |
| `media` | Currently playing track | `max_length` |
| `paused` | Shown (in red) when hotkeys are paused | `color` |

The `active_window` widget displays the application icon of the currently
focused window. It extracts the icon from the running process and renders
it as a bitmap in the bar. When no window is focused or the icon cannot be
determined, the widget is hidden automatically.

The `media` widget is meant to show the track that is playing ("Artist -
Title"), with long titles truncated to `max_length` characters (default 40)
and an ellipsis, and to hide itself when nothing plays. It does not read
anything on Windows yet: the query to the Windows media controls (GSMTC)
blocks the daemon, so for now the widget always stays hidden. You can keep
it in `bar.toml` for when that is solved.

```toml
[[right]]
type = "media"
icon = "\uF001"       # music note icon
max_length = 40       # truncate long titles
```

### Example: Full Bar

```toml
[[left]]
type = "workspaces"

[[left]]
type = "active_window"

[[left]]
type = "layout"

[[center]]
type = "clock"
format = "%H:%M"

[[right]]
type = "media"
icon = "\uF001"
max_length = 40

[[right]]
type = "cpu"

[[right]]
type = "ram"

[[right]]
type = "date"
format = "%a %d %b"
```

The `paused` widget is auto-hidden when hotkeys are active and shown in red
when they are paused. It is included in the default config and is added
automatically to existing `bar.toml` files on daemon startup (along with any
other new default widgets from future versions).

## The Mouse

The bar takes clicks and the wheel. Workspaces, the layout and monocle change
on the bar's own monitor:

| Where | What | Result |
|-------|------|--------|
| A workspace number | left click | Go to that workspace (every monitor in global mode) |
| On the bar | wheel | The workspace before (up, left) or after (down, right) |
| The layout widget | left click | Cycle the layout |
| The layout widget | right click | Toggle monocle |
| The **PAUSED** mark | left click | Resume the hotkeys |
| Anything you can click | hover | The pointer shows a hand |

A click on another monitor's bar that changes its workspace, layout or
monocle moves focus to that monitor, as if you had gone there. One that runs
`retile` or `toggle-pause`, such as a click on **PAUSED**, leaves focus where
it is. Either way the pointer stays where you left it. The turns of a
burst of the wheel, or of a touchpad's swipe, add up, and once the wheel
goes quiet the bar switches as many workspaces on as they come to. A slow
high-resolution wheel gets there too, a fraction at a time. Stepping past
the last workspace stops there, or goes round to the first with
`[workspaces] wrap = true` in `config.toml`. Clicks keep working while the
hotkeys are paused, which is how the **PAUSED** mark resumes them.

The bar never takes focus. Over a maximized or fullscreen window, managed or
not, it lets the pointer through, so the window's own title bar and tabs
work as before. Elsewhere it takes clicks only over what you can click, each
over its whole column of the bar, and where it draws something: with the
default transparent background, a window that reaches under the bar, such as
a Snap half, still gets its clicks there between the bar's labels and pills.
With any `background_opacity` above 0, the bar takes clicks over all of its
strip.

```toml
[mouse]
enabled = true           # false keeps the bar click-through
scroll = "workspaces"    # "workspaces" or "none"
```

What a click on a widget does is up to you. Each `[[mouse.click]]` entry
binds a button, `left`, `right` or `middle`, on a widget, named by its type,
to an action, named as in `keybindings.toml`, in place of what that button
does there now, and a later entry for the same widget and button wins. It
works on every widget of that type:

```toml
[[mouse.click]]
widget = "clock"
button = "middle"
action = "retile"

[[mouse.click]]
widget = "layout"
button = "left"
action = "goto-workspace-next"   # instead of cycling the layout

[[mouse.click]]
widget = "layout"
button = "right"
action = "none"                  # a right click does nothing
```

`action = "none"` turns a click off, a default included, and a widget with
no click left takes none. Deleting an entry brings back the default for that
widget and button.

Only actions on the bar's own monitor or on the whole daemon can be bound:
`cycle-layout`, `toggle-monocle`, `goto-workspace-N`, `goto-workspace-next`
and `goto-workspace-prev` act on the monitor of the bar you click, and
`retile` and `toggle-pause` on the whole daemon. An entry for another action,
such as `close-focused`, which would close a window on another monitor, or
one with a typo or a key left out, is ignored, and `mosaico doctor` says
which and why; the rest of `bar.toml` still applies. The right-click of a
workspace number runs what you bind on the `workspaces` widget, and its left
click always goes to that workspace.

The wheel needs "Scroll inactive windows when I hover over them" on in the
Windows mouse settings, as it is by default: with it off, Windows sends the
wheel to the window that has focus, and the bar never does.

## Work Area

The bar reserves space at the top of the monitor. The tiling layout
automatically accounts for the bar height so windows do not overlap it.

## Hot-Reload

Changes to `bar.toml` are automatically applied while the daemon is running.
The bar is recreated with the new settings and all monitors are retiled.

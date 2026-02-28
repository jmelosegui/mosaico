# Status Bar

Mosaico includes a configurable status bar that sits at the top of each
monitor. It is configured in `~/.config/mosaico/bar.toml`.

## Configuration

```toml
enabled = true
height = 28
monitor = "all"      # "all", "primary", or 0-based index

[colors]
background = "base"      # Named Catppuccin color or hex
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

Colors can be specified as hex values (`"#1e1e2e"`) or as named Catppuccin
colors (`"base"`, `"text"`, `"blue"`, etc.). See [Theming](theming.md) for
the full list of named colors.

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

The `active_window` widget displays the application icon of the currently
focused window. It extracts the icon from the running process and renders
it as a bitmap in the bar. When no window is focused or the icon cannot be
determined, the widget is hidden automatically.

The `media` widget shows the currently playing track ("Artist - Title")
using the Windows media transport controls API (GSMTC). It works with
Spotify, browser media (YouTube, SoundCloud, etc.), VLC, and any app that
integrates with Windows media controls. When nothing is playing, the
widget is hidden automatically. Long titles are truncated to `max_length`
characters (default 40) with an ellipsis. Some third-party media apps
may not register with GSMTC and will not be detected.

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

## Work Area

The bar reserves space at the top of the monitor. The tiling layout
automatically accounts for the bar height so windows do not overlap it.

## Hot-Reload

Changes to `bar.toml` are automatically applied while the daemon is running.
The bar is recreated with the new settings and all monitors are retiled.

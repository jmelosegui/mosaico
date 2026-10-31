# Configuration

Mosaico uses TOML configuration files stored in `~/.config/mosaico/`. All
settings have sensible defaults, so configuration files are entirely optional.

Run `mosaico init` to generate fully-commented template files.

## Configuration Files

| File | Purpose | Hot-Reload |
|------|---------|------------|
| `config.toml` | Layout, borders, mouse, window transitions, theme, logging | Yes |
| `keybindings.toml` | Keyboard shortcuts | No (restart required) |
| `rules.toml` | Community window rules (auto-downloaded) | On startup |
| `user-rules.toml` | Personal window rule overrides | Yes |
| `bar.toml` | Status bar settings | Yes |

## `config.toml`

Controls layout, borders, the mouse, window transitions, logging, and theme:

```toml
[layout]
default = "bsp"  # Default layout algorithm: "bsp", "vertical-stack", "three-column"
gap = 8          # Pixel gap between windows (0-200)
ratio = 0.5      # BSP split ratio (0.1-0.9)
hiding = "cloak" # How windows hide on workspace switch: "cloak", "hide", "minimize"

[workspaces]
# How a workspace switch is applied across monitors.
# "per-monitor" (default): only the focused monitor switches.
# "global": all monitors switch in lockstep, like Windows virtual desktops.
mode = "per-monitor"
# Whether stepping past the last workspace, with goto-workspace-next or the
# wheel over the bar, goes back to the first.
wrap = false

[workspaces.layouts]
# Available: "bsp", "vertical-stack", "three-column"
1 = "three-column"    # Override layout for workspace 1
3 = "vertical-stack"  # Override layout for workspace 3
# Workspaces without an entry use [layout].default

[borders]
width = 4              # Border thickness in pixels (0-32)
corner_style = "small" # "square", "small", or "round"

[borders.colors]
focused = "#00b4d8"    # Color of the focused window in tiled layouts
monocle = "#2d6a4f"    # Color of the focused window in monocle mode
unfocused = "#6c7086"  # Color drawn around unfocused tiled windows;
                       # set to "none" to disable unfocused borders

[mouse]
follows_focus = true         # Move the pointer to the window that takes focus
focus_follows_mouse = false  # Focus the window under the pointer without a click
drag_to_swap = true          # Drop a dragged tiled window on another to swap them
resize_splits = true         # Drag the edge between two tiles to resize them

[animations]
enabled = true            # Move windows to their new place over time (on by default)
duration = 180            # How long a transition takes, in milliseconds (0-1000)
easing = "ease-out-cubic" # linear, ease-out-quad, ease-out-cubic, ease-in-out-cubic,
                          # ease-out-expo, or "cubic-bezier(x1, y1, x2, y2)"
follow_system = true      # Snap while Windows turns animations off,
                          # and in a remote session

[theme]
name = "catppuccin"  # catppuccin, rose-pine, tokyo-night
flavor = "mocha"     # catppuccin: mocha | macchiato | frappe | latte
                     # rose-pine:  main | moon | dawn
                     # tokyo-night: night | storm | day

[logging]
enabled = false    # Enable file logging
level = "info"     # Log level: debug, info, warn, error
max_file_mb = 10   # Max log file size before rotation
```

The `[animations]` section sets up window transitions, which are on by
default. See [Window Transitions](animations.md) for what moves, the easings
and troubleshooting.

The `[mouse]` section ties the pointer to focus. With `follows_focus`, on by
default, the pointer moves to the center of the window that gets focus when
you move focus from the keyboard, switch workspaces or open a window, and it
moves along when you move the focused window. Clicking a window leaves the
pointer where it is. With `focus_follows_mouse`, off by default, resting the
pointer on a tiled window focuses it without a click: after three checks of
the pointer, 100 ms apart, as long as the window is on the monitor that has
focus. A click or a scroll on the status bar never moves the pointer, and
while you drag a window, the pointer stays with you, either way.

With `drag_to_swap`, on by default, dropping a tiled window you drag by its
title bar on another tile swaps the two, and dropping it on another monitor
puts it there next to the tile under the pointer, with a preview showing
where it will land while you drag, and the windows there making room for
it, out of monocle once you drop it. With `false`, a dropped window goes
back to its slot, or joins the end of the other monitor's windows. See
[Dragging Windows](tiling-layout.md#dragging-windows).

With `resize_splits`, on by default, dragging the edge between two tiles
resizes them: the windows on the other side follow the edge as you drag
it, the split between them moves to where you let go, and a corner where
two splits meet moves both. Each workspace keeps where you put its splits,
for each layout. With `false`, a window you resize goes back to its slot.
See [Resizing Tiles](tiling-layout.md#resizing-tiles).

## `keybindings.toml`

Maps key combinations to actions. See
[Keyboard Bindings](keybindings.md) for the full reference.

```toml
[[keybinding]]
action = "focus-down"
key = "J"
modifiers = ["alt"]
```

## `bar.toml`

Configures the status bar. See [Status Bar](status-bar.md) for details.

```toml
enabled = true
height = 28
monitors = []            # 0-based monitor indices; empty for all of them

[mouse]
enabled = true           # Take clicks and the wheel; false keeps the bar click-through
scroll = "workspaces"    # What scrolling over the bar does: "workspaces" or "none"

[[mouse.click]]          # What a click on a widget runs, over the defaults
widget = "clock"         # A widget's type
button = "middle"        # "left", "right" or "middle"
action = "retile"        # An action on a monitor or the whole daemon

[colors]
background = "#1e1e2e"   # Hex, or empty for the theme's own
foreground = "#cdd6f4"
accent = "blue"          # A color the theme names

[[left]]
type = "workspaces"

[[right]]
type = "clock"
format = "%H:%M"

[[right]]
type = "cpu"
```

## `rules.toml` and `user-rules.toml`

Window rules decide which windows are tiled, which float above the tiles
and which are left alone. Community rules are
downloaded automatically to `rules.toml` on daemon startup. Add personal
overrides in `user-rules.toml` (user rules take priority).

See [Window Rules](window-rules.md) for details.

```toml
# user-rules.toml — your personal overrides
[[rule]]
match_title = "My Private Tool"
action = "ignore"
```

## Partial Files

All configuration fields have defaults. You only need to include the
settings you want to change. A file containing only:

```toml
[layout]
gap = 12
```

will use defaults for everything else.

## Validation

Out-of-range values are automatically clamped to safe boundaries rather
than rejected:

| Setting | Valid Range |
|---------|------------|
| `gap` | 0 -- 200 |
| `ratio` | 0.1 -- 0.9 |
| `border width` | 0 -- 32 |
| `corner_style` | `"square"`, `"small"`, `"round"` |
| `hiding` | `"cloak"`, `"hide"`, `"minimize"` |
| `default` | `"bsp"`, `"vertical-stack"`, `"three-column"` |
| `[animations] duration` | 0 -- 1000 (milliseconds) |

An `[animations] easing` is never clamped. It must be `"linear"`,
`"ease-out-quad"`, `"ease-out-cubic"`, `"ease-in-out-cubic"`,
`"ease-out-expo"`, or `"cubic-bezier(x1, y1, x2, y2)"` with each number from
0 to 1. Any other value makes the whole `config.toml` invalid: a running
mosaico keeps the settings it has, and one that starts uses the defaults.

## Hot-Reload

Changes to `config.toml`, `user-rules.toml`, and `bar.toml` are
automatically detected and applied while the daemon is running. The daemon
waits for the system to report a change in the config directory, then
reloads each of these files whose modification time changed.

- **config.toml** -- layout gap/ratio, hiding behaviour, border settings,
  mouse settings, window transitions and theme are reloaded. Windows are
  retiled immediately. Hiding changes take effect on the next workspace
  switch. Windows already moving keep going, unless the reload moves them
  elsewhere or turns transitions off.
- **user-rules.toml** -- both rule sets are re-merged, your rules first.
  New rules apply to new windows. Managed windows the new rules no longer
  manage are released at once, and their monitor is retiled. A window on
  another workspace that the new rules stop tiling appears on screen right
  away, where it was, no longer tiled. A window the new rules would manage
  is picked up the next time it gets focus.
- **bar.toml** -- the status bar is recreated with updated settings and
  colors.
- **rules.toml** -- community rules, not watched: updated only on daemon
  startup.
- **keybindings.toml** -- **not** hot-reloaded. Changes require a daemon
  restart (`mosaico stop && mosaico start`).

Only valid configurations are applied. If a change to `config.toml` or
`bar.toml` introduces a parse error, it is logged
(`config.toml invalid, skipping: ...`) and the current configuration is kept
until the file changes again. A `user-rules.toml` that does not parse counts
as no user rules, as it does at startup, so the community rules apply alone
until it is fixed.

# Window Borders

Mosaico draws a colored border overlay around every visible tiled
window. The focused window is highlighted with the focused (or monocle)
color; every other tiled window on the active workspace gets a muted
unfocused border so tile boundaries stay readable -- especially with
small gaps. Unfocused borders can be disabled if you prefer a minimal
look.

## Configuration

Border settings are in `config.toml`:

```toml
[borders]
width = 4              # Border thickness in pixels (0-32)
corner_style = "small" # "square", "small", or "round"

[borders.colors]
focused = "#00b4d8"    # Color for focused window in tiled layouts
monocle = "#2d6a4f"    # Color for focused window in monocle mode
unfocused = "#6c7086"  # Color drawn around unfocused tiled windows
```

- **width** -- thickness of the border in pixels. Set to `0` to disable
  every border.
- **corner_style** -- controls both the border overlay shape and the
  DWM corner preference for tiled windows (Windows 11 only; ignored
  on Windows 10).

  | Value | Border overlay | DWM (Windows 11) |
  |-------|---------------|------------------|
  | `"square"` | Sharp corners (0 px) | `DONOTROUND` |
  | `"small"` | Subtle rounding (8 px) | `ROUNDSMALL` (~4 px) |
  | `"round"` | Standard rounding (16 px) | `ROUND` (~8 px) |

- **borders.colors.focused** -- the border color for the focused window
  during normal tiling. Accepts hex colors or named theme colors
  (see [Theming](theming.md)).
- **borders.colors.monocle** -- the border color when monocle mode is
  active.
- **borders.colors.unfocused** -- the border color drawn around every
  unfocused tiled window. Set to `"none"` to disable unfocused borders
  entirely (only the focused window will have a border in that case).
  An empty string falls back to the active theme's muted-gray default.

## Behavior

- Every tiled window on every monitor's active workspace gets its own
  border overlay.
- Changing focus recolors the previously focused border to the
  unfocused color and the new focused border to the focused color, in
  one update.
- Switching workspaces hides the outgoing windows' borders and shows
  the incoming windows' borders.
- In monocle mode only the focused window's border renders; all other
  tiled borders on that monitor are hidden until monocle is toggled
  off.
- When the focused window is maximized, its border is hidden so it
  does not overflow off-screen, and unfocused borders on the same
  monitor are also hidden because they are visually covered.
- Borders are click-through (`WS_EX_TRANSPARENT`) and never interfere
  with mouse interaction.

## Using Named Colors

With a theme active, you can use named colors:

```toml
[borders.colors]
focused = "blue"
monocle = "green"
unfocused = "mauve"
```

Empty values fall back to the theme's defaults: blue for focused,
green for monocle, and a muted gray (Catppuccin Overlay0) for
unfocused.

## Disabling Unfocused Borders Only

If you want the focus highlight without borders on every other window:

```toml
[borders.colors]
unfocused = "none"
```

This keeps `focused` and `monocle` rendering as before and skips
unfocused borders entirely. Use `width = 0` instead if you want every
border off.

## Hot-Reload

Border settings are hot-reloaded. Changes to `width`, `corner_style`,
and any `[borders.colors]` entry take effect immediately, including
flipping `unfocused = "none"` on or off.

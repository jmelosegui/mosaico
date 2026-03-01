# Focus Borders

Mosaico draws a colored border overlay around the currently focused window,
making it easy to identify which window has focus.

## Configuration

Border settings are in `config.toml`:

```toml
[borders]
width = 4              # Border thickness in pixels (0-32)
corner_style = "small" # "square", "small", or "round"
focused = "#00b4d8"    # Color for focused window
monocle = "#2d6a4f"    # Color for monocle mode
```

- **width** -- thickness of the border in pixels. Set to `0` to disable
  the focus border entirely.
- **corner_style** -- controls both the border overlay shape and the
  DWM corner preference for tiled windows (Windows 11 only; ignored
  on Windows 10).

  | Value | Border overlay | DWM (Windows 11) |
  |-------|---------------|------------------|
  | `"square"` | Sharp corners (0 px) | `DONOTROUND` |
  | `"small"` | Subtle rounding (8 px) | `ROUNDSMALL` (~4 px) |
  | `"round"` | Standard rounding (16 px) | `ROUND` (~8 px) |

- **focused** -- the border color during normal tiling. Accepts hex colors
  or named Catppuccin colors (see [Theming](theming.md)).
- **monocle** -- the border color when monocle mode is active.

## Behavior

- The border automatically follows focus as you navigate between windows
- It is a click-through overlay that does not interfere with your
  interaction with the focused window
- The border sits on top of all windows (topmost)
- It is excluded from tiling (it is invisible to the tiling manager)

## Using Named Colors

With a theme active, you can use named colors:

```toml
[borders]
focused = "blue"
monocle = "green"
```

## Hot-Reload

Border settings are hot-reloaded. Changes to `width`, `corner_style`,
`focused`, and `monocle` colors take effect immediately.

# Focus Borders

Mosaico draws a colored border overlay around the currently focused window,
making it easy to identify which window has focus.

## Configuration

Border settings are in `config.toml`:

```toml
[borders]
width = 4              # Border thickness in pixels (0-32)
focused = "#00b4d8"    # Color for focused window
monocle = "#2d6a4f"    # Color for monocle mode
```

- **width** -- thickness of the border in pixels. Set to `0` to disable
  the focus border entirely.
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

Border settings are hot-reloaded. Changes to `width`, `focused`, and
`monocle` colors take effect immediately.

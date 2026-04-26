# Window Border Overlays

Mosaico draws a colored rectangular border around every visible tiled
window. The focused window is highlighted with the focused (or monocle)
color; other tiled windows on every monitor's active workspace are drawn
in a muted unfocused color so tile boundaries stay readable, especially
with small gaps. Each border is a transparent overlay window that uses
`UpdateLayeredWindow` with per-pixel alpha for flicker-free rendering.

Unfocused borders can be turned off entirely (see
[Configuration](#configuration) below) when a minimal look is preferred.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-windows/src/border.rs` | `Border` struct, `Color` struct, rendering logic |
| `crates/mosaico-core/src/config/mod.rs` | `BorderConfig` for color and width settings |

### Key Types

- `Border` -- wraps a layered `HWND` overlay window
- `Color` -- `r`, `g`, `b` fields with `from_hex()` parser

## Border Window

The border is a Win32 window with special styles:

- `WS_POPUP` -- no title bar or frame
- `WS_EX_LAYERED` -- supports per-pixel alpha transparency
- `WS_EX_TOOLWINDOW` -- hidden from taskbar and Alt+Tab
- `WS_EX_TOPMOST` -- always on top of other windows
- `WS_EX_TRANSPARENT` -- click-through (mouse events pass to windows below)

Window class `MosaicoBorder` is registered once via `std::sync::Once`.

## Rendering

`Border::render(rect, color, width)`:

1. Creates a DIB section (device-independent bitmap) matching the border
   dimensions
2. Fills border pixels with the specified color at full opacity
3. Fills interior pixels with transparent (all zero bytes)
4. Calls `UpdateLayeredWindow` with `AC_SRC_ALPHA` to apply the bitmap as
   the window content

This approach avoids using GDI drawing functions and provides pixel-perfect
borders with full alpha support.

### Render-Before-Position

`Border::show()` calls `render()` first, then `SetWindowPos`. This ordering
is intentional -- rendering before repositioning avoids a visual glitch where
a stale bitmap from the previous size would briefly appear at the new position.

## Methods

- `Border::new()` -- creates the overlay window (initially hidden)
- `Border::show(rect, color, width)` -- renders the border bitmap and
  positions the overlay window over the target rectangle
- `Border::hide()` -- hides the overlay window via `ShowWindow(SW_HIDE)`

## Color

The `Color` struct represents an RGB color:

- `Color::from_hex(s)` -- parses `"#RRGGBB"` or `"RRGGBB"` format strings
- Default focused color: theme-resolved (Mocha: `#89b4fa` blue)
- Default monocle color: theme-resolved (Mocha: `#a6e3a1` green)
- Default unfocused color: theme-resolved (Mocha: `#6c7086` muted gray)

The literal `"none"` on `unfocused` disables unfocused borders;
`BorderColors::unfocused_enabled()` reports the toggle state.

## Integration

The `TilingManager` keeps a `HashMap<usize, Border>` keyed by `hwnd`,
with one entry per visible tiled window. The lifecycle is unified
through a single entry point:

- `update_border()` -- called after every layout, focus, workspace, or
  display event. It snapshots the active workspace on every monitor,
  drops borders for windows that are no longer visible, creates
  borders for new windows, and recolors each surviving border based on
  whether it owns focus.
- `hide_border()` -- hides every overlay without dropping them
  (used on pause and shutdown).

Color selection per window:

- The focused window uses the configured `focused` color, or the
  `monocle` color if its monitor is in monocle mode.
- Every other window on the active workspace of every monitor uses the
  `unfocused` color, unless that color is set to the `"none"` sentinel
  (in which case unfocused windows have no border).
- Maximized focused windows skip rendering, and on the focused monitor
  only the monocle window renders while monocle mode is active.

## Configuration

In `config.toml`:

```toml
[borders]
width = 4              # Border thickness in pixels (0-32)
corner_style = "small" # "square", "small", or "round"

[borders.colors]
focused = "#00b4d8"    # Color for the focused window in tiled layouts
monocle = "#2d6a4f"    # Color when monocle mode is active
unfocused = "#6c7086"  # Color drawn around unfocused tiled windows;
                       # set to "none" to disable unfocused borders
```

Setting `width = 0` disables every border. To keep the focused border
but hide unfocused ones, set `unfocused = "none"`.

### Rounded Corners

The `corner_style` field controls both the border overlay shape and the
tiled windows themselves.

| Value | Border overlay | DWM (Windows 11) |
|-------|---------------|------------------|
| `"square"` | Sharp corners (0 px) | `DONOTROUND` |
| `"small"` | Subtle rounding (8 px) | `ROUNDSMALL` (~4 px) |
| `"round"` | Standard rounding (16 px) | `ROUND` (~8 px) |

On Windows 10, the DWM attribute is silently ignored — only the border
overlay is affected.

## Design Decisions

- **`UpdateLayeredWindow`** is used instead of a regular painted window to
  avoid requiring a message pump for `WM_PAINT` handling. The border window
  renders its content atomically via a DIB bitmap.
- **`WS_EX_TRANSPARENT`** makes the border click-through, so it never
  interferes with interaction on the focused window.
- **`WS_EX_TOOLWINDOW`** hides the border from taskbar and Alt+Tab,
  preventing it from appearing as an application to the user.
- **`WINEVENT_SKIPOWNPROCESS`** in the event loop ensures the border window
  does not trigger tiling events -- without this, creating/moving the border
  would cause recursive tiling loops.
- **DIB section rendering** gives full control over per-pixel alpha without
  depending on GDI drawing primitives, which have limited alpha support.
- **One overlay per visible window**, keyed by `hwnd` in a `HashMap`.
  Borders are created lazily when a window is first added to a visible
  workspace and dropped (via `Drop`, which calls `DestroyWindow`) when
  the window leaves it. This avoids scanning a Vec on every update at
  the cost of one layered window per tile.

## Tests

`Color::from_hex()` is covered by unit tests for both `#RRGGBB` and
`RRGGBB` formats. Theme tests assert that `border_focused`,
`border_monocle`, and `border_unfocused` are distinct per flavor and
match their Catppuccin palette values.

# Focus Border Overlay

Mosaico displays a colored rectangular border around the focused window to
provide clear visual feedback. The border is a transparent overlay window that
uses `UpdateLayeredWindow` with per-pixel alpha for flicker-free rendering.

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
- Default focused color: `#00b4d8` (cyan)
- Default monocle color: `#2d6a4f` (dark green)

## Integration

The `TilingManager` creates one `Border` instance at startup and manages its
lifecycle:

- `update_border()` -- called on focus changes; shows the border around the
  focused window with the appropriate color (normal or monocle)
- `hide_border()` -- called when focus leaves all managed windows

The border color changes between the configured `focused` color and `monocle`
color depending on whether monocle mode is active on the focused monitor.

## Configuration

In `config.toml`:

```toml
[borders]
width = 4              # Border thickness in pixels (0-32)
focused = "#00b4d8"    # Color for normal focused window
monocle = "#2d6a4f"    # Color when monocle mode is active
```

Setting `width = 0` effectively disables the border.

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
- **Single border instance** is reused and repositioned rather than creating
  a new window for each focus change, avoiding window creation overhead.

## Tests

1 unit test for `Color::from_hex()` covering both `#RRGGBB` and `RRGGBB`
formats.

# Phase 16: Active Window Icon Widget

**Status:** Complete

**Goal:** Add a new bar widget that displays the icon of the currently focused
window, positioned between the workspace indicators and the layout widget.

## Overview

The active window icon widget extracts the application icon from the focused
window's HWND and renders it as a small bitmap in the status bar. This gives
users an at-a-glance visual indicator of which application is currently
focused, complementing the existing workspace and layout widgets.

## Reference Design

```
[1] 2 3 4 5 6 7 8  [icon]  BSP
^^^^^^^^^^^^^^^^^^^^  ^^^^  ^^^
workspace numbers    active  layout
                     window
                     icon
```

The icon sits between the last workspace number and the layout pill. It is
rendered as a square bitmap scaled to fit the bar height (minus vertical
padding), typically 16x16 or 20x20 depending on bar height.

## Architecture

### Data Flow

1. `TilingManager` tracks `focused_window: Option<usize>` (the HWND)
2. `bar_states()` includes the focused HWND per monitor in `BarState`
3. The `active_window` widget module extracts the icon from the HWND at
   render time using Win32 APIs
4. The icon is drawn directly onto the DIB buffer as BGRA pixels

### New Files

```
crates/
  mosaico-windows/
    src/
      bar/
        widgets/
          active_window.rs   # Icon extraction and rendering
```

### Modified Files

```
crates/
  mosaico-core/
    src/
      config/
        bar.rs               # Add WidgetConfig::ActiveWindow variant
  mosaico-windows/
    src/
      bar/
        widgets/
          mod.rs             # Register ActiveWindow in dispatch
      tiling/
        mod.rs               # Add focused_hwnd to bar_states()
```

### Key Types

- `WidgetConfig::ActiveWindow` -- new enum variant with `enabled` and `icon`
  fields (following existing pattern)
- `BarState::focused_hwnd` -- `Option<usize>` carrying the focused window
  handle to the renderer

### Icon Extraction Strategy

Win32 provides multiple ways to get a window's icon, tried in priority order:

1. **`SendMessageTimeout(WM_GETICON, ICON_SMALL2)`** -- 16x16 icon set by
   the application (best quality for small sizes)
2. **`SendMessageTimeout(WM_GETICON, ICON_SMALL)`** -- small icon fallback
3. **`GetClassLongPtrW(GCLP_HICONSM)`** -- small icon registered with the
   window class
4. **`SendMessageTimeout(WM_GETICON, ICON_BIG)`** -- large icon (will be
   scaled down)
5. **`GetClassLongPtrW(GCLP_HICON)`** -- large class icon fallback

Use `SendMessageTimeout` (not `SendMessage`) with a short timeout (~50ms)
to avoid blocking the render thread if a window is hung.

### Icon Rendering

Once an `HICON` is obtained:

1. Create a temporary memory DC and a 32-bit BGRA DIB section at the target
   icon size (e.g., 16x16 or 20x20)
2. Call `DrawIconEx` onto the DIB to rasterize the icon
3. Copy the BGRA pixels from the icon DIB into the bar's main DIB buffer
   at the correct (x, y) position
4. Handle premultiplied alpha: icon pixels may have alpha; ensure they
   composite correctly over the bar background
5. Clean up the temporary DC and DIB

If no icon is available (e.g., no focused window, or icon extraction fails),
the widget renders nothing and takes zero horizontal space, letting the
layout widget shift left seamlessly.

### Widget Behavior

- **No focused window**: widget is invisible (0 width)
- **Focused window has no icon**: widget is invisible (0 width)
- **Normal case**: renders a square icon with `item_gap` spacing
- The widget is purely visual -- no text, no pill background
- Updates on every `BarState` refresh (focus changes, workspace switches)

## Configuration

### `bar.toml`

The widget appears in the `[[left]]` array between workspaces and layout:

```toml
[[left]]
type = "workspaces"

[[left]]
type = "active_window"

[[left]]
type = "layout"
icon = "\uF009"
```

### Widget Configuration

| Type | Description | Extra Fields |
|------|-------------|--------------|
| `active_window` | Icon of the currently focused window | `enabled` (default: true) |

The `icon` field from `WidgetConfig` is unused for this widget since it
renders a bitmap, not text. It exists only for serde consistency with other
variants.

## Integration Points

- **BarState**: needs a new `focused_hwnd: Option<usize>` field populated
  by `TilingManager::bar_states()`. The focused HWND comes from
  `self.focused_window` for the focused monitor, `None` for other monitors.
- **Widget dispatch**: `draw_left()` in `widgets/mod.rs` must handle
  `WidgetConfig::ActiveWindow` as a special case (like `Workspaces`), since
  it renders a bitmap, not a text pill.
- **DrawCtx**: the `buf` field (raw BGRA pixel slice) is already accessible,
  so the icon widget can blit pixels directly without new infrastructure.
- **GDI resources**: the icon extraction creates temporary DCs per render.
  These are lightweight and cleaned up immediately. No persistent state is
  needed between renders since the focused window can change at any time.

## Design Decisions

- **Extract icon at render time**: rather than caching icons, extract fresh
  each render. Icons rarely change, and `SendMessageTimeout` is fast (~1ms).
  Caching adds complexity (invalidation on window close, focus change, etc.)
  with minimal benefit at a 1-second refresh rate.
- **SendMessageTimeout over SendMessage**: prevents the bar render from
  blocking indefinitely if the focused application is hung (Not Responding).
  A 50ms timeout ensures the bar stays responsive.
- **No pill background**: the icon is a raw bitmap, not text in a pill.
  This keeps it visually distinct from text widgets and avoids awkward
  sizing of a pill around a tiny square.
- **Invisible when empty**: rather than showing a placeholder, the widget
  collapses to zero width when there's no icon. This keeps the bar clean
  and avoids visual noise on empty workspaces.
- **Platform-specific code only**: icon extraction uses Win32 APIs and lives
  entirely in `mosaico-windows`. The `WidgetConfig` variant in `mosaico-core`
  is platform-agnostic (just config), and `BarState::focused_hwnd` is a plain
  `usize` that each platform can interpret as its native handle type.
- **Icon sizing**: the icon is scaled to `bar_height - 2 * pill_top` to
  match the vertical space used by pills, keeping visual alignment consistent.

## Tasks

- [x] Add `WidgetConfig::ActiveWindow` variant to `bar.rs` with `enabled`
      and `icon` fields (matching existing pattern)
- [x] Add `ActiveWindow` to `WidgetConfig::icon()` and `enabled()` match arms
- [x] Insert `ActiveWindow` in `default_left_widgets()` between Workspaces
      and Layout
- [x] Add `focused_hwnd: Option<usize>` to `BarState` struct and `Default`
- [x] Populate `focused_hwnd` in `TilingManager::bar_states()` -- set to
      `self.focused_window` for the focused monitor, `None` for others
- [x] Create `active_window.rs` in `bar/widgets/` with:
  - [x] `extract_icon(hwnd)` -- uses SHGetFileInfoW via PID -> exe path
  - [x] `draw()` -- extracts icon, creates temp DIB, calls DrawIconEx,
        blits BGRA pixels into bar buffer, returns new X position
- [x] Register `pub mod active_window` in `widgets/mod.rs`
- [x] Add `WidgetConfig::ActiveWindow` arm to `draw_left()` dispatch
      (special-case like `Workspaces`, calls `active_window::draw()`)
- [x] Add `WidgetConfig::ActiveWindow` arm to `widget_text()` (returns
      empty string) and `draw_pill_right()` if needed
- [x] Build with `cargo build`
- [x] Run `cargo clippy --workspace` and fix warnings
- [x] Run `cargo fmt --all`
- [x] Run `cargo test` and fix any failures
- [x] Manual test: verify icon appears for various apps (Chrome, Explorer,
      Terminal, VS Code), disappears on empty workspace, handles hung
      windows gracefully
- [x] Update `docs/status-bar.md` with ActiveWindow widget documentation

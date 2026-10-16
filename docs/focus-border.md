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
| `crates/mosaico-core/src/decoration.rs` | `Decorations`, the contract for borders and window corners |
| `crates/mosaico-core/src/tiler/decoration_effects.rs` | `apply_decoration_effect`, which carries out border and corner effects |
| `crates/mosaico-core/src/tiler/plan_borders.rs` | Keeping one border per window in a step, and where it goes |
| `crates/mosaico-core/src/tiler/last_drawn.rs` | `LastDrawn`, the style a border was last drawn with |
| `crates/mosaico-core/src/tiler/applier_borders.rs` | The borders of moving windows, drawn once each window is in place |
| `crates/mosaico-core/src/config/color.rs` | `Color` struct and its hex parser |
| `crates/mosaico-core/src/config/mod.rs` | `BorderConfig` for color and width settings |
| `crates/mosaico-windows/src/decorations.rs` | `Win32Decorations`, the Win32 implementation |
| `crates/mosaico-windows/src/border.rs` | `Border` struct: creating, placing and hiding the overlay |
| `crates/mosaico-windows/src/border/render.rs` | Painting the border bitmap |
| `crates/mosaico-windows/src/border/stack.rs` | Putting the overlay just behind its window |
| `crates/mosaico-windows/src/border/class.rs` | The `MosaicoBorder` window class and its window procedure |

### Key Types

- `Decorations` -- the core trait the tiler's border and corner effects go through
- `Win32Decorations` -- one overlay per bordered window, a `Border` and the
  `LastDrawn` style its bitmap shows, plus the DWM corner preference
- `LastDrawn` -- in `mosaico-core`, decides whether a border needs drawing
  again or only putting back in place
- `Border` -- wraps a layered `HWND` overlay window
- `Color` -- `r`, `g`, `b` fields with `from_hex()` parser, in `mosaico-core`

## Contract

Borders and window corners reach the platform through the `Decorations`
trait in `mosaico-core`:

```rust
pub trait Decorations {
    fn show_border(&mut self, window: WindowId, style: &BorderStyle) -> Result<()>;
    fn hide_border(&mut self, window: WindowId) -> Result<()>;
    fn drop_border(&mut self, window: WindowId) -> Result<()>;
    fn forget_drawn(&mut self);
    fn set_corners(&mut self, window: WindowId, corners: CornerStyle) -> Result<()>;
    fn reset_corners(&mut self, window: WindowId) -> Result<()>;
}
```

The contract, in the trait's docs, spells out what the tiler relies on:

- A border surrounds the window's visible bounds, the same bounds
  `WindowControl` places windows at, with its stroke outside them.
- A border never takes focus or input, never shows in the taskbar or the
  window switcher, and never causes a window event.
- Each time it is drawn, a border goes directly behind its window.
- Hiding or dropping the border of a window that has none does nothing.
- Drawing a border fails only when the border cannot be created. A window
  that closed in the meantime is not an error.
- A platform may keep what it drew of a border, and only move it when
  the border is drawn again the same way. After `forget_drawn`, the next
  drawing of each border starts from scratch.
- A platform that cannot draw borders, or cannot change the corners of
  windows, returns `Error::Unsupported`.

`tiler::apply_decoration_effect` carries out the `Border`, `DropBorder`,
`RedrawBorders`, `Decorate` and `Undecorate` effects through it. A border
that cannot be created is logged as a warning, unless the platform has no
borders at all; other failures, such as a window that closed in the
meantime, are logged for debugging.

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
  positions the overlay window over the target rectangle; it fails when
  the bitmap does not reach the overlay, which keeps the one it had and
  is still put in place
- `Border::restack(rect, width, target)` -- puts the overlay in its
  window's topmost band, then just behind its window at `rect`, and only
  then shows it, with the bitmap it already has, so a hidden border never
  shows where it was hidden
- `Border::hide()` -- hides the overlay window via `ShowWindow(SW_HIDE)`

`Win32Decorations` remembers the style each overlay was last rendered with,
in a `LastDrawn` from `mosaico-core`, which makes the decision: a border
shown again with a rect of the same size, and the same color, width and
radius, is only restacked at its rect, because its window may have moved
in the z-order or to another slot, and a hidden one is shown again by
that. A layered window keeps the bitmap `UpdateLayeredWindow` gave it when
it is moved or hidden. Rendering takes tens of milliseconds for a large
tile, about 30 ms for half a 4K screen in a release build.
`forget_drawn` starts a new `LastDrawn` for every overlay, so each one
renders its bitmap the next time it is drawn.

## Color

The `Color` struct (`mosaico_core::config::Color`) represents an RGB
color. The bar draws with it too.

- `Color::from_hex(s)` -- parses `"#RRGGBB"` or `"RRGGBB"` format strings:
  six hex digits in either case, after any leading `#`. Anything else,
  such as a value with a `+` sign or a character outside ASCII, is not a
  color.
- Default focused color: theme-resolved (Mocha: `#89b4fa` blue)
- Default monocle color: theme-resolved (Mocha: `#a6e3a1` green)
- Default unfocused color: theme-resolved (Mocha: `#6c7086` muted gray)

The literal `"none"` on `unfocused` disables unfocused borders;
`BorderColors::unfocused_enabled()` reports the toggle state.

The tiler parses the configured value when it decides how a border is
drawn, and sends the platform a `Color` in `BorderStyle`. A value that is
still not a hex color after theme resolution, such as a name the theme
does not know or `"none"` on `focused` or `monocle`, is drawn in
`BorderColors::FALLBACK` (`#00b4d8`).

## Integration

The tiler decides every border and emits it as an effect; in
`mosaico-windows`, the `Win32Decorations` that `Win32Platform` hands the
tiling manager keeps a
`HashMap<WindowId, Overlay>` keyed by window id, with one entry per
visible tiled window: its `Border` and the `LastDrawn` style its bitmap
shows. The lifecycle is unified through a single entry point:

- `update_border()` -- called after every layout, focus, workspace, or
  display event. It snapshots the active workspace on every monitor,
  drops borders for windows that are no longer visible, creates
  borders for new windows, and recolors each surviving border based on
  whether it owns focus.
- `hide_borders()` -- hides every border without dropping it, when the
  tiler stops.

One step often decides a window's border more than once: when the layout
places the window, when `update_border()` runs after the layout, and again
after focus moves. The step draws only the last decision for each window,
once, in the place of its first border after the last `Place` or
`DropBorder` of that window that comes before that decision. So each
border lands right after its own window, never before the
placement it was decided for, and a hide or drop wins over an earlier
draw.

The user's retile and a display change that changes anything draw every
border from scratch. They start with `RedrawBorders`, which has the
platform forget how it drew each border: what it kept may be lost by
then, as when the display changes, and a retile is how the user sets a
stale desktop right. The daemon runs the retile action after reading the
work areas again. Other steps draw from scratch only the borders that
look different, even when they lay out every window again: reading the
work areas again, which a taskbar change reports several times,
reserving the bar and reloading the config.

While a burst of moves is held until input goes quiet, each window's
border is held with its move: only the latest style, drawn around the slot
the move lands in. That holds even when a later step, such as a focus
change, decided the border from where the window still was.

### Borders of Moving Windows

Drawing a border can take tens of milliseconds for a large window, too long
for a frame of a transition, so a border is never drawn while its window
moves (`tiler/applier_borders.rs`). A `Border` effect for a moving window
only replaces the style that is drawn once the window is in place, around
the rect it ends at; a hide happens at once, since it costs nothing. A
window whose size changes on its way has its border hidden as its motion
starts, since the border could not follow it, and drawn once at the end.
A window whose motion stops on the way, as when it is hidden or the user
grabs it, gets no border drawn for that motion: what the tiler decides for
it next is drawn as it comes. Nor does one the tiler lets go of while it
moves: it is placed where it was going, and the step drops its border.
One a display change stops keeps the style until it is placed at its
target, after the rebuild, and has it drawn there, unless the tiler placed
it or took it out of view meanwhile; a `Border` effect for it meanwhile
replaces the style, as for a moving window. `DropBorder` forgets the style
a motion kept, so a border dropped meanwhile is not drawn again.

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

Windows 10 does not have the DWM attribute, so there only the border
overlay is affected: changing corners is reported as unsupported, and
logged at debug level.

The tiler gives the configured corners to the windows it takes in, and
gives the system's corners back only to the windows it gave them to:
when they hide or close, when a rule reload lets go of them, and when
the tiler stops. Menus and windows the rules leave alone keep the
corners they chose.

Two gaps remain. A window adopted after being dragged does not get the
configured corners until the config is reloaded. A managed window that
is minimized keeps the tiler's corners when the tiler stops, and when
the rules change to leave it alone while it is minimized.

## Design Decisions

- **`UpdateLayeredWindow`** is used instead of a regular painted window to
  avoid requiring a message pump for `WM_PAINT` handling. The border window
  renders its content atomically via a DIB bitmap.
- **`WS_EX_TRANSPARENT`** makes the border click-through, so it never
  interferes with interaction on the focused window.
- **`WS_EX_TOOLWINDOW`** hides the border from taskbar and Alt+Tab,
  preventing it from appearing as an application to the user.
- **`WINEVENT_SKIPOWNPROCESS`** on the window event hooks ensures the border window
  does not trigger tiling events -- without this, creating/moving the border
  would cause recursive tiling loops.
- **DIB section rendering** gives full control over per-pixel alpha without
  depending on GDI drawing primitives, which have limited alpha support.
- **One overlay per visible window**, keyed by window id in a `HashMap`.
  Borders are created lazily when a window is first added to a visible
  workspace and dropped (via `Drop`, which calls `DestroyWindow`) when
  the window leaves it. This avoids scanning a Vec on every update at
  the cost of one layered window per tile.

## Tests

`Color::from_hex()` is covered by unit tests in `config/color_tests.rs`:
every spelling of six hex digits, with or without `#` and in any case,
and values with a multibyte character, which are not colors.
`config/color_digit_tests.rs` checks that a `+` sign is not a hex digit.
Theme tests assert that `border_focused`,
`border_monocle`, and `border_unfocused` are distinct per flavor and
match their Catppuccin palette values.

In `mosaico-core`, `tiler/tests/decoration_effects_tests.rs` drives
`apply_decoration_effect` against a platform that records what it is
asked to draw, `tiler/tests/border_tests.rs` checks the colors borders are
drawn in, including the fallback and `"none"`, and
`tiler/tests/corner_tests.rs` checks which windows get their corners back.
`tiler/tests/border_once_tests.rs` checks that opening a window, a retile,
a focus change and a workspace switch decide each border once, after its
window's last placement, and `tiler/tests/border_order_tests.rs` that each
border is drawn right after its own window. `tiler/tests/plan_borders_tests.rs` and
`tiler/tests/coalesce_border_tests.rs` check which border is kept, in a
step and while a burst of moves is held. `tiler/tests/held_border_tests.rs`
checks that a focus change or a border refresh before a held move lands
still draws each border around its window's new slot, in the right color.

`tiler/tests/last_drawn_tests.rs` checks when a border needs drawing
again. `tiler/tests/border_redraw_tests.rs` runs the tiling manager on a
platform that keeps what it drew, and checks that a retile, also after
reading the work areas again as the daemon runs it, and a display change
draw every border from scratch, while reading the work areas again,
reserving the bar and a display change that changes nothing draw none
again, and a focus change draws only the borders that change. In
`mosaico-windows`, `border/show_tests.rs` checks that restacking a hidden
border, as drawing it again the same way does, shows it again, and only
once it is at its new rect, with overlays of its own off screen, and
`decorations_tests.rs` that
forgetting how the borders were drawn renders every one again.

# Window Management

Mosaico changes windows through a platform contract in `mosaico-core`,
`WindowControl`, and reads them through `tiler::Desktop`. Both identify windows
by `WindowId`. `mosaico-windows` implements them for Win32 `HWND` handles,
which includes invisible frame compensation and DPI awareness.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/window.rs` | `WindowControl` trait |
| `crates/mosaico-core/src/tiler/desktop.rs` | `Desktop` trait, the read side |
| `crates/mosaico-core/src/tiler/window_effects.rs` | `apply_window_effect()`, which carries out window effects |
| `crates/mosaico-windows/src/window.rs` | Win32 `Window` struct wrapping an `HWND` |
| `crates/mosaico-windows/src/window_control.rs` | `Win32WindowControl`, the Win32 `WindowControl` |
| `crates/mosaico-windows/src/frame.rs` | `visible_rect()`, `border_offset()`, `BorderOffset` |
| `crates/mosaico-windows/src/enumerate.rs` | `enumerate_windows()` |
| `crates/mosaico-windows/src/dpi.rs` | `enable_dpi_awareness()` |

### Key Types

- `WindowControl` (trait, core) -- changes the platform makes on request
- `Desktop` (trait, core) -- what the tiler reads about windows and monitors
- `mosaico_core::Result<T>` -- `Result<T, mosaico_core::Error>`, the error
  type of every platform contract
- `Window` (struct, windows) -- wraps `HWND`
- `Win32WindowControl` (struct, windows) -- implements `WindowControl`
- `BorderOffset` -- `left`, `top`, `right`, `bottom` offsets for frame
  compensation

## Window Control

The tiler never changes a window itself. It returns effects, and the ones that
change a window or move the pointer reach the platform through `WindowControl`:

```rust
pub trait WindowControl {
    fn place(&self, window: WindowId, rect: Rect) -> Result<()>;
    fn pre_place(&self, window: WindowId, rect: Rect) -> Result<()>;
    fn repaint(&self, window: WindowId) -> Result<()>;
    fn focus(&self, window: WindowId) -> Result<()>;
    fn close(&self, window: WindowId) -> Result<()>;
    fn minimize(&self, window: WindowId) -> Result<()>;
    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;
    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;
    fn reveal(&self, window: WindowId) -> Result<()>;
    fn move_cursor(&self, x: i32, y: i32) -> Result<()>;
}
```

The contract, in the trait's docs, spells out what callers rely on:

- Rects are a window's visible bounds in desktop coordinates, without invisible
  resize borders or shadows.
- Placing a maximized window takes it out of maximized state, and a placement
  is never reported as the user moving the window.
- Pre-placement, for windows that exist but are not shown yet, never waits for
  the application, even one that never handles its messages.
- A minimize is reported back as a `Minimized` event, which is what takes the
  window out of the layout. Hiding with `Cloak` is not reported as a hide.
- Failures are returned as errors, never panics. Callers log them and carry on.

`tiler::apply_window_effect` makes the decisions that are left until an effect
is applied, against the `Desktop` as it is then: a window already in place is
left alone unless it is maximized, a placement is followed by a repaint even
when the move fails, and when the pointer follows focus it goes to the center
of the window. Borders and window corners are not window changes and stay with
the platform's applier. `debug move` places windows through
`WindowControl::place` as well.

Window ids typed by the user, such as the `--window` of `debug move`, are read
with `WindowId::parse`, which takes the `0x` hex form ids are printed in, or
decimal.

A failed OS call comes back as `Error::Platform`, which names what was being
done and keeps the native error as its source. `mosaico-windows` cannot
implement `From<windows::core::Error>` for the core error, so it wraps each
call with the crate-private `OrPlatform::or_platform("move the window")`
helper in `crates/mosaico-windows/src/error.rs`.

## Win32 Implementation

The `Window` struct in `mosaico-windows` wraps an `HWND` and queries the OS
lazily. `Win32WindowControl` and the tiling module's `Win32Desktop` build on it
to implement the core contracts.

### Constructors

- `Window::new(hwnd: HWND)` -- wraps an existing `HWND` (inside the crate)
- `Window::from_id(id: WindowId)` -- the window a core id refers to

### Methods

- `title()` -- `GetWindowTextW` to retrieve the window title
- `class()` -- `RealGetWindowClassW` to retrieve the window class name
- `rect()` -- delegates to `frame::visible_rect()` for the DWM-aware
  bounding box
- `is_visible()` -- `IsWindowVisible`
- `is_maximized()`, `is_minimized()`, `is_cloaked()`, `owner()`,
  `process_id()`, `is_elevated()`, `exe_path()`, `app_window_caption()` --
  the queries behind `Win32Desktop`
- `hwnd()` -- returns the underlying `HWND`

### Window Control on Win32

| Method | Win32 |
|--------|-------|
| `place` | `set_rect()`: clears `WS_MAXIMIZE`, then `SetWindowPos` with frame compensation (see below) |
| `pre_place` | `move_async()`: the same move with `SWP_ASYNCWINDOWPOS`, which posts the request instead of sending it |
| `repaint` | `invalidate()`: `RedrawWindow` with aggressive flags, then realigns a Chromium window's GPU surface |
| `focus` | `SetForegroundWindow` |
| `close` | `PostMessageW` with `WM_CLOSE` |
| `minimize` | `ShowWindow(SW_MINIMIZE)` |
| `hide` | `Cloak`: ImmersiveShell cloak; `Hide`: `ShowWindow(SW_HIDE)`; `Minimize`: `ShowWindow(SW_MINIMIZE)` |
| `show` | `Cloak`: uncloak; otherwise `ShowWindow(SW_SHOWNOACTIVATE)` |
| `reveal` | uncloak, then `ShowWindow(SW_SHOW)` |
| `move_cursor` | `SetCursorPos` |

`ShowWindow` reports whether the window was visible before, not whether the
call worked, so `minimize`, `hide`, `show` and `reveal` first check with
`IsWindow` that the window still exists. A cloak or uncloak that fails, for a
window the shell has no view of for example, comes back as an error.

## Window Positioning

`set_rect()` uses `SetWindowPos` with specific flags:

- `SWP_NOZORDER` -- do not change Z-order
- `SWP_NOACTIVATE` -- do not activate the window
- `SWP_NOSENDCHANGING` -- suppress `WM_WINDOWPOSCHANGING` so windows cannot
  reject the resize
- `SWP_NOCOPYBITS` -- do not copy client area bits (prevents visual artifacts)
- `SWP_FRAMECHANGED` -- conditionally added for Chromium/Firefox windows

### Invisible Frame Compensation

On Windows 10/11, windows have invisible extended borders that are part of the
window rect but not visible on screen. Without compensation, tiled windows
would have gaps between them.

`frame::border_offset()` computes the difference between:
- `GetWindowRect` -- includes invisible borders
- `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` -- visible bounds only

The offset is applied when calling `SetWindowPos` to ensure pixel-perfect
positioning of visible window content.

`frame::visible_rect()` returns the DWM extended frame bounds (the visible
rect), falling back to `GetWindowRect` if DWM is unavailable.

## Window Enumeration

`enumerate_windows()` uses `EnumWindows` with a callback to discover all
visible application windows. It filters to:

- Visible windows (`IsWindowVisible`)
- Non-minimized windows
- Application windows (has `WS_CAPTION`, not `WS_EX_TOOLWINDOW`)

Returns a `Vec<Window>` used during daemon initialization to discover existing
windows on the desktop.

## DPI Awareness

`enable_dpi_awareness()` calls
`SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` to
ensure accurate pixel coordinates on mixed-DPI multi-monitor setups.

- Must be called before any Win32 window/monitor APIs
- Called at the start of `daemon::run()` and in debug commands
- Failure is silently ignored (the DPI context may already be set via a
  manifest)

Without DPI awareness, the OS would virtualize coordinates, causing incorrect
window positioning on high-DPI monitors.

## Design Decisions

- `WindowControl` and `Desktop` take window ids rather than window objects, so
  the tiler, its tests and the CLI never hold a platform type, and a fake
  desktop needs no handles.
- Their methods take `&self` because the OS owns window state -- Rust
  mutability semantics don't apply to external resource handles.
- Invisible frame compensation is essential for Windows 10/11 where DWM adds
  invisible shadow borders to all windows.
- `SWP_NOSENDCHANGING` prevents windows from overriding the requested size,
  which is critical for a tiling manager that must enforce exact positions.
- `SWP_FRAMECHANGED` is only added for Chromium/Firefox because it can cause
  other applications to enforce minimum size constraints, breaking layouts.
- `RedrawWindow` with `RDW_INVALIDATE | RDW_ALLCHILDREN` flags forces
  GPU-composited applications (browsers, Electron apps) to repaint correctly
  after repositioning.
- DPI awareness V2 (per-monitor) is chosen over V1 because it provides
  correct coordinates for each monitor independently in multi-monitor setups
  with different DPI scales.

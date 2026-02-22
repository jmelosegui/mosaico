# Window Management

Mosaico uses a two-layer window abstraction: `mosaico-core` defines a
platform-agnostic `Window` trait, and `mosaico-windows` implements it for
Win32 `HWND` handles. This layer also handles invisible frame compensation
and DPI awareness.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/window.rs` | `Window` trait, `WindowResult<T>` type alias |
| `crates/mosaico-windows/src/window.rs` | Win32 `Window` struct implementing the trait |
| `crates/mosaico-windows/src/frame.rs` | `visible_rect()`, `border_offset()`, `BorderOffset` |
| `crates/mosaico-windows/src/enumerate.rs` | `enumerate_windows()` |
| `crates/mosaico-windows/src/dpi.rs` | `enable_dpi_awareness()` |

### Key Types

- `Window` (trait, core) -- platform-agnostic window interface
- `WindowResult<T>` -- `Result<T, Box<dyn std::error::Error>>`
- `Window` (struct, windows) -- wraps `HWND`
- `BorderOffset` -- `left`, `top`, `right`, `bottom` offsets for frame
  compensation

## Window Trait

The `Window` trait in `mosaico-core` defines the platform-agnostic interface:

```rust
pub trait Window {
    fn title(&self) -> WindowResult<String>;
    fn class(&self) -> WindowResult<String>;
    fn rect(&self) -> WindowResult<Rect>;
    fn set_rect(&self, rect: &Rect) -> WindowResult<()>;
    fn invalidate(&self) -> WindowResult<()>;
    fn is_visible(&self) -> bool;
}
```

Note: `set_rect()` takes `&self` (not `&mut self`) because mutation happens
on the OS side via the window handle.

## Win32 Implementation

The `Window` struct in `mosaico-windows` wraps an `HWND` and implements the
trait plus additional Windows-specific methods.

### Constructors

- `Window::new(hwnd: HWND)` -- wraps an existing `HWND`
- `Window::from_raw(handle: usize)` -- creates from a raw handle value

### Trait Methods

- `title()` -- `GetWindowTextW` to retrieve the window title
- `class()` -- `GetClassNameW` to retrieve the window class name
- `rect()` -- delegates to `frame::visible_rect()` for the DWM-aware
  bounding box
- `set_rect()` -- positions the window with frame compensation (see below)
- `invalidate()` -- `RedrawWindow` with aggressive flags for GPU-composited
  apps
- `is_visible()` -- `IsWindowVisible`

### Additional Methods

- `hwnd()` -- returns the underlying `HWND`
- `set_foreground()` -- `SetForegroundWindow` to bring the window to front
- `is_app_window()` -- checks `WS_CAPTION` and rejects `WS_EX_TOOLWINDOW`
- `needs_frame_changed()` -- detects Chromium (`Chrome_WidgetWin_1`) and
  Firefox (`MozillaWindowClass`) windows that need `SWP_FRAMECHANGED`

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

- `Window` trait uses `&self` for `set_rect()` because the OS owns window
  state -- Rust mutability semantics don't apply to external resource handles.
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

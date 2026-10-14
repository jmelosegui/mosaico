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
| `crates/mosaico-core/src/window/behind.rs` | `Behind`: where `put_behind` puts a window, given which of the two are always on top |
| `crates/mosaico-core/src/hiding.rs` | `HidingEvents`, `HidingSupport`: the ways of hiding a platform has, and what each reports |
| `crates/mosaico-core/src/tiler/desktop.rs` | `Desktop` trait, the read side |
| `crates/mosaico-core/src/tiler/window_effects.rs` | `apply_window_effect()`, which carries out window effects |
| `crates/mosaico-windows/src/window.rs` | Win32 `Window` struct wrapping an `HWND`, and its basic queries |
| `crates/mosaico-windows/src/window/place.rs` | `set_rect()` and `invalidate()`: placing a window and repainting it |
| `crates/mosaico-windows/src/window/pre_place.rs` | `move_async()`: moving a window that has not been shown yet |
| `crates/mosaico-windows/src/window/process.rs` | The owning process: `process_id()`, `is_elevated()`, `exe_path()` |
| `crates/mosaico-windows/src/window/kind.rs` | `app_window_caption()`: whether a window is an application window |
| `crates/mosaico-windows/src/window_control.rs` | `Win32WindowControl`, the Win32 `WindowControl` |
| `crates/mosaico-windows/src/window_control/hiding.rs` | Hiding, showing, revealing and minimizing windows for `Win32WindowControl` |
| `crates/mosaico-windows/src/window_control/stacking.rs` | `put_behind` for `Win32WindowControl`, within each window's `WS_EX_TOPMOST` band |
| `crates/mosaico-windows/src/frame.rs` | `visible_rect()`, `border_offset()`, `BorderOffset` |
| `crates/mosaico-core/src/tiler/enumerator.rs` | `Enumerator`, the contract for listing windows and monitors |
| `crates/mosaico-windows/src/enumerate.rs` | `WindowsEnumerator`, `enumerate_windows()` |
| `crates/mosaico-core/src/system/setup.rs` | `ProcessSetup`, the setup every process runs first |
| `crates/mosaico-windows/src/setup.rs` | `WindowsSetup`, the Windows `ProcessSetup` |
| `crates/mosaico-windows/src/dpi.rs` | `enable_dpi_awareness()` |

### Key Types

- `WindowControl` (trait, core) -- changes the platform makes on request
- `Behind` (enum, core) -- where `put_behind` puts a window: directly behind
  the other when both share an always-on-top layer, nowhere otherwise
- `HidingEvents` (struct, core) -- whether hiding a window one way reports it
  hidden or minimized, and showing it again reports it restored
- `HidingSupport` (struct, core) -- the platform's `HidingEvents` for every
  way of hiding, `None` for the ways it does not have
- `Desktop` (trait, core) -- what the tiler reads about windows and monitors
- `mosaico_core::Result<T>` -- `Result<T, mosaico_core::Error>`, the error
  type of every platform contract
- `Window` (struct, windows) -- wraps `HWND`; private to `mosaico-windows`,
  so other crates reach windows only through the core contracts
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
    fn put_behind(&self, window: WindowId, other: WindowId) -> Result<()>;
    fn close(&self, window: WindowId) -> Result<()>;
    fn minimize(&self, window: WindowId) -> Result<()>;
    fn hide(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;
    fn show(&self, window: WindowId, how: HidingBehaviour) -> Result<()>;
    fn hiding_events(&self, how: HidingBehaviour) -> Option<HidingEvents>;
    fn reveal(&self, window: WindowId) -> Result<()>;
    fn move_cursor(&self, x: i32, y: i32) -> Result<()>;
}
```

The contract, in the trait's docs, spells out what callers rely on:

- Rects are a window's visible bounds in desktop coordinates, without invisible
  resize borders or shadows.
- Placing a maximized window takes it out of maximized state, and a placement
  is never reported as the user moving the window.
- Placing a window hidden with `hide` moves it and keeps it out of view, so it
  can be shown in place. Placing a minimized window leaves it minimized and
  need not change where it is restored to, so the tiler restores a window
  before it places it.
- Pre-placement, for windows that exist but are not shown yet, never waits for
  the application, even one that never handles its messages.
- `put_behind(window, other)` puts `window` behind `other` in the stacking
  order, within its own always-on-top layer, without activating, moving or
  resizing either. Neither window's layer changes: `window` goes directly
  behind `other` when they share a layer, and stays where it is otherwise
  (`Behind`), which is behind `other` already when only `other` is always on
  top. When only `window` is, it stays in front of `other`. It is the only way
  to change which window is in front without focusing one, and it fails when
  either window is gone.
- A minimize is reported back as a `Minimized` event, which is what takes the
  window out of the layout.
- `hiding_events` says which ways of hiding the platform has, and whether
  hiding a window each way reports it hidden or minimized and showing it again
  reports it restored. `hide` and `show` return `Error::Unsupported` for a way
  the platform does not have. The tiler hides windows only in ways the
  platform has, unless it has none, and takes the hide and restore reports it
  says to expect for its own. Focus reports for a window hidden moments ago
  are dropped whatever it says (see
  [workspaces.md](workspaces.md#hidden-window-tracking)).
- Failures are returned as errors, never panics. Callers log them and carry on.

`tiler::apply_window_effect` makes the decisions that are left until an effect
is applied, against the `Desktop` as it is then: a window already in place is
left alone unless it is maximized, a placement is followed by a repaint even
when the move fails, and when the pointer follows focus it goes to the center
of the window. Borders and window corners are not window changes: they go
through `Decorations` and `tiler::apply_decoration_effect` (see
[focus-border.md](focus-border.md#contract)). `debug move` places windows
through `WindowControl::place` as well.

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
  the queries behind `Win32Desktop` and `WindowsEnumerator::details`
- `hwnd()` -- returns the underlying `HWND` (inside the crate)

### Window Control on Win32

| Method | Win32 |
|--------|-------|
| `place` | `set_rect()`: clears `WS_MAXIMIZE`, then `SetWindowPos` with frame compensation (see below) |
| `pre_place` | `move_async()`: the same move with `SWP_ASYNCWINDOWPOS`, which posts the request instead of sending it |
| `repaint` | `invalidate()`: `RedrawWindow` with aggressive flags, then realigns a Chromium window's GPU surface |
| `focus` | `SetForegroundWindow` |
| `put_behind` | `SetWindowPos` with `other` as the window to insert after and `SWP_NOMOVE`, `SWP_NOSIZE` and `SWP_NOACTIVATE`, only when both or neither are `WS_EX_TOPMOST`: inserted after a window of the other band, a window moves into that band |
| `close` | `PostMessageW` with `WM_CLOSE` |
| `minimize` | `ShowWindow(SW_MINIMIZE)` |
| `hide` | `Cloak`: ImmersiveShell cloak; `Hide`: `ShowWindow(SW_HIDE)`; `Minimize`: `ShowWindow(SW_MINIMIZE)` |
| `show` | `Cloak`: uncloak; otherwise `ShowWindow(SW_SHOWNOACTIVATE)` |
| `hiding_events` | Every way. Cloaking fires no hide or minimize event; `SW_HIDE` fires `EVENT_OBJECT_HIDE`; `SW_MINIMIZE` fires `EVENT_SYSTEM_MINIMIZESTART`, and showing the window again `EVENT_SYSTEM_MINIMIZEEND` |
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

Windows are listed through the core `Enumerator` contract
(`mosaico-core/src/tiler/enumerator.rs`). `Enumerator::windows()` returns
every top-level window that is shown and not minimized, front to back,
except tool palettes and floating toolbars, windows owned by another window
(dialogs, property sheets) and modal dialogs. Windows without a title bar are
listed, since the window rules may opt them in. It makes no other judgement:
windows the system keeps out of view and windows mosaico cannot move are
listed too, and `Desktop::candidate` rules them out. The daemon tiles the
windows it accepts in this order at startup, and `mosaico debug list` prints
them.

On Windows, `WindowsEnumerator` calls `enumerate_windows()`, which uses
`EnumWindows` (top-level windows in Z order, topmost first) and keeps:

- Visible windows (`IsWindowVisible`)
- Non-minimized windows (`IsIconic`)
- Application windows: not `WS_EX_TOOLWINDOW`, no owner, not
  `WS_EX_DLGMODALFRAME` (`Window::app_window_caption()`). `WS_CAPTION` is
  not required

`Enumerator::details()` describes one window for `debug list`, and gives
`debug events` its window titles: title, class, executable path, whether it
has a title bar, rect, state, monitor and the invisible frame border
(`frame::border_offset()`).

## DPI Awareness

Every mosaico process, the daemon and each CLI command alike, first runs the
platform's setup, `ProcessSetup::init()`, which `main` calls through
`platform::init()` before any command. Its contract is that afterwards the
positions and sizes of windows and monitors are in the same units in every
mosaico process, whatever the display scaling, so `debug list`, `debug move`
and `doctor` show and use the coordinates the daemon tiles with.

On Windows, `WindowsSetup` calls `enable_dpi_awareness()`, which calls
`SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)` to
ensure accurate pixel coordinates on mixed-DPI multi-monitor setups.

- Must be called before any Win32 window/monitor APIs, which is why `main`
  runs it before any command
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

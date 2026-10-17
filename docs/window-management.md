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
| `crates/mosaico-core/src/frames/control.rs` | `FrameControl`: moving windows through the frames of their transitions, and whether a change is posted behind them |
| `crates/mosaico-core/src/frames/frame.rs` | `Frame`, `FrameOutcome`: a window's rect at a moment of its transition, and what `place_frames` found of each window |
| `crates/mosaico-core/src/frames/table.rs` | `FrameTable`, `PENDING_LIMIT`, `ASYNC_GRACE`: what a platform keeps about the frames it posts, and whether the next one goes |
| `crates/mosaico-core/src/frames/table_kept.rs` | What a `FrameTable` keeps about one window, looked up with its owner every time, and `kept_for` |
| `crates/mosaico-core/src/frames/table_queue.rs` | `posts_behind`: whether a window's other changes are posted behind what may be queued for it, and the notes of what was |
| `crates/mosaico-core/src/frames/probe.rs` | `FrameProbe`: how the table reads a platform's windows and posts their moves |
| `crates/mosaico-core/src/frames/posted.rs` | `LastPost`: the last frame, or the last of anything, posted to a window, whether the window has been seen to move since, and whether the frame holds the next one back |
| `crates/mosaico-core/src/hiding.rs` | `HidingEvents`, `HidingSupport`: the ways of hiding a platform has, and what each reports |
| `crates/mosaico-core/src/tiler/desktop.rs` | `Desktop` trait, the read side |
| `crates/mosaico-core/src/tiler/window_effects.rs` | `apply_window_effect()`, which carries out window effects |
| `crates/mosaico-windows/src/window.rs` | Win32 `Window` struct wrapping an `HWND`, and its basic queries |
| `crates/mosaico-windows/src/window/place.rs` | `set_rect()` and `redraw()`: placing a window and repainting it |
| `crates/mosaico-windows/src/window/posted_place.rs` | What a posted placement reads of a window and does to it: `is_shown()`, `post_restore()`, `post_move()` |
| `crates/mosaico-windows/src/window/pre_place.rs` | `move_async()`: moving a window that has not been shown yet |
| `crates/mosaico-windows/src/window/process.rs` | The owning process: `process_id()`, `is_elevated()`, `exe_path()` |
| `crates/mosaico-windows/src/window/kind.rs` | `app_window_caption()`: whether a window is an application window |
| `crates/mosaico-windows/src/window_control.rs` | `Win32WindowControl`, the Win32 `WindowControl` |
| `crates/mosaico-windows/src/window_control/frames.rs` | `FrameControl` for `Win32WindowControl`, and the `FrameProbe` its `FrameTable` reads and moves windows through |
| `crates/mosaico-windows/src/window_control/ordered.rs` | Making a window's other changes in order with its frames, as the `FrameTable` says, and noting what was posted |
| `crates/mosaico-windows/src/window_control/posted_placement.rs` | How a placement posted behind what was posted to a window ends a maximized state, and the borders it moves the window with |
| `crates/mosaico-windows/src/window_control/frames_post.rs` | Where `GetWindowRect` reads a window, `IsHungAppWindow`, and the `SetWindowPos` that posts a frame |
| `crates/mosaico-windows/src/window_control/frames_dpi.rs` | Where a window that is not per-monitor DPI aware can be put: frames mapped to its own pixels and back; the frame widths at a DPI |
| `crates/mosaico-windows/src/window_control/frames_kept.rs` | What is kept about a window with its frames, and its borders at the DPI of its display now |
| `crates/mosaico-windows/src/window_control/hiding.rs` | Hiding, showing, revealing and minimizing windows for `Win32WindowControl` |
| `crates/mosaico-windows/src/window_control/stacking.rs` | `put_behind` for `Win32WindowControl`, within each window's `WS_EX_TOPMOST` band |
| `crates/mosaico-windows/src/frame.rs` | `visible_rect()`, `border_offset()`, `BorderOffset` |
| `crates/mosaico-windows/src/system_settings.rs` | `animations_allowed()`: the Windows animation setting, and whether the session is remote |
| `crates/mosaico-core/src/tiler/enumerator.rs` | `Enumerator`, the contract for listing windows and monitors |
| `crates/mosaico-windows/src/enumerate.rs` | `WindowsEnumerator`, `enumerate_windows()` |
| `crates/mosaico-core/src/system/setup.rs` | `ProcessSetup`, the setup every process runs first |
| `crates/mosaico-windows/src/setup.rs` | `WindowsSetup`, the Windows `ProcessSetup` |
| `crates/mosaico-windows/src/dpi.rs` | `enable_dpi_awareness()` |

### Key Types

- `WindowControl` (trait, core) -- changes the platform makes on request
- `FrameControl` (trait, core) -- moving windows through the frames of their
  transitions, whether a change made to a window is posted behind them
  (`posts_behind`), and forgetting the windows that need it no longer once
  transitions are off (`settle_responding`); every `WindowControl` is one,
  unsupported by default
- `Behind` (enum, core) -- where `put_behind` puts a window: directly behind
  the other when both share an always-on-top layer, nowhere otherwise
- `Frame` (struct, core) -- a window, the rect it should have at a moment of
  its transition, and whether it is the first frame of its motion
- `FrameOutcome` (enum, core) -- what `place_frames` found of a window:
  `Handled { reached }`, `Pending`, `Maximized`, `Minimized` or `Gone`
- `FrameTable<K>` (struct, core) -- what a platform keeps about the frames it
  posts to each window, `K` being what the platform keeps of its own, and the
  decisions on whether the next frame goes and whether a change is posted, the
  same for every platform
- `FrameProbe` (trait, core) -- what a `FrameTable` reads of a platform's
  windows, and how it posts their moves
- `PENDING_LIMIT` (const, core) -- 250 ms: how long a frame posted to a window
  that has not been seen to move since holds the next one back
- `ASYNC_GRACE` (const, core) -- 5 s: how long after something was posted to
  a window its other changes are posted too, behind it, even once it has been
  seen to move since
- `LastPost` (struct, core) -- the last frame, or the last of anything, posted
  to a window: when, where the window was then, and whether it has been seen
  anywhere else since
- `HidingEvents` (struct, core) -- whether hiding a window one way reports it
  hidden or minimized, and showing it again reports it restored
- `HidingSupport` (struct, core) -- the platform's `HidingEvents` for every
  way of hiding, `None` for the ways it does not have
- `Desktop` (trait, core) -- what the tiler reads about windows and monitors,
  and whether windows may be animated
- `mosaico_core::Result<T>` -- `Result<T, mosaico_core::Error>`, the error
  type of every platform contract
- `Window` (struct, windows) -- wraps `HWND`; private to `mosaico-windows`,
  so other crates reach windows only through the core contracts
- `Win32WindowControl` (struct, windows) -- implements `WindowControl` and
  `FrameControl`; the `FrameTable` of the frames of transitions is the only
  thing it keeps between calls
- `BorderOffset` -- `left`, `top`, `right`, `bottom` offsets for frame
  compensation

## Window Control

The tiler never changes a window itself. It returns effects, and the ones that
change a window or move the pointer reach the platform through `WindowControl`:

```rust
pub trait WindowControl: FrameControl {
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

pub trait FrameControl {
    // Unsupported by default. Clears `outcomes`, then fills it, one per frame.
    fn place_frames(&self, frames: &[Frame], outcomes: &mut Vec<FrameOutcome>) -> Result<()>;
    // False by default: whether a change made to `window` now is posted.
    fn posts_behind(&self, window: WindowId) -> bool;
    // Nothing by default: forgets the windows posted to no longer.
    fn settle_responding(&self);
}
```

Every `WindowControl` is a `FrameControl` too (`frames/control.rs`), so the
frames of transitions reach the platform through the same object, and a
platform that cannot move windows frame by frame keeps the default. The
caller keeps the `outcomes` buffer from one frame to the next, so the
outcomes are not allocated again for every frame.

What each frame does is decided in core, by a `FrameTable` the platform keeps,
so every platform decides the same way. The platform only reads its windows
and posts their moves through a `FrameProbe`, and gives the table the time
from its own clock. For each frame, in this order: a window that is gone is
`Gone`; a minimized or maximized one is reported so and left alone; one that
another owner has now, and whose motion is not starting over, is `Gone`; the
first frame of a motion reads the window again (`FrameProbe::keep`), and
starts over for a window whose owner changed, a new window given the id of an
old one; a window whose application is not responding is `Pending`; a frame
posted less than `PENDING_LIMIT` ago to a window that has not been seen to
move since holds the new one back, `Pending`; a frame the window can only take
at the rect it is at is not needed, `Handled`; and any other is posted,
`Handled`. A post the system refuses holds nothing back, and is `Handled` too.
The table also says when a window's other changes have to be posted behind
what may be queued for it (`posts_behind`), is told what was posted
(`posted`, `posted_show_state`) and what was made at once
(`show_state_made`), and keeps what a posted placement needs: what the
platform read with the window's frames (`kept_for`), and whether the window is
to be in view once what was posted lands (`in_view_once_landed`).

The contract, in the traits' docs, spells out what callers rely on:

- Rects are a window's visible bounds in desktop coordinates, without invisible
  resize borders or shadows.
- Placing a maximized window takes it out of maximized state, whether the
  placement is posted or made at once, and a placement is never reported as
  the user moving the window. One exception: a placement posted to a window
  out of view, or to be once what was posted lands, may leave it maximized
  while its application does not respond, or while the change that takes it
  out of view is still on its way. A later show then brings it back as if it
  had not been placed: a hidden window at the normal bounds it had before it
  was maximized.
- Placing a window hidden with `hide` moves it and keeps it out of view, so it
  can be shown in place. Placing a minimized window leaves it minimized and
  need not change where it is restored to, so the tiler restores a window
  before it places it.
- Pre-placement never waits for the application, even one that never handles
  its messages. It moves windows that exist but are not shown yet, and a window
  that is already shown when it is taken in, ahead of the layout. The move may
  land at once, later, or not at all, and moves made this way to one window
  land in the order they were made. A maximized window may stay maximized.
- `FrameControl::place_frames` moves windows to the frames of their
  transitions without waiting for their applications, and reports one
  `FrameOutcome` per frame, in order. A frame posted less than `PENDING_LIMIT`
  (250 ms) ago to a window that has not been seen to move since holds the next
  one back: `Pending`, and nothing posted. A slow application gets at most one
  frame that often, never a backlog that grows with the frame rate. Nothing
  depends on knowing whether a frame landed: any move since the post lets the
  next one go, wherever the window went, and so does the limit, for a window
  that rounds its size, puts it back, or is sent back where it was. A window
  whose application is not responding is `Pending`, and gets nothing posted.
- `Handled` means the frame was posted, or was not needed: once mapped to what
  the window can take (its own pixels, when it does not follow the display's
  scale), it asks for where the window is. It also covers a post the system
  refused, which holds nothing back. The first frame of a motion reads the
  window again. A window with another owner starts over with it, and is
  `Gone` for any other frame.
- `place_frames` never raises, focuses, shows or hides a window or changes
  whether one is maximized or minimized: such a window is reported and left
  alone. A frame queued when the user or the system maximizes or minimizes a
  window may land after that; the next call reports the window `Maximized` or
  `Minimized`, and the caller leaves it alone. A frame that lands is reported
  as the window's position changing (`LocationChanged`), never as the user
  moving it. A platform without frames returns `Error::Unsupported`, the
  default; asking with no frames tells which it is.
- Placements, repaints, `put_behind` and changes of a window's show state
  (`minimize`, `hide`, `show`, `reveal`) keep their order with the frames
  posted before them. One made while something posted to the window may still
  be queued, as `FrameTable::posts_behind` says, is posted behind it instead
  of made at once: less than `ASYNC_GRACE` (5 s) after anything was posted to
  the window; after that, for as long as the window has not been seen to move
  since the last thing posted to it, a frame or any other change, which may
  still be queued or may have been taken without a move; and, once the
  window was given a frame, while its application does not respond, since a
  change made at once would wait for it. A window never given a frame is
  changed at once, responding or not, as it always was, so with animations
  off from startup every change is, and the calls made to one whose
  application does not respond wait for it, as they always did. Once they
  are turned off, `FrameControl::settle_responding`, which the tiler calls
  while they stay off, forgets every window something was last posted to
  more than `ASYNC_GRACE` before and whose application responds, which is
  then changed at once again; one whose application hangs stays posted to
  while it does. Hiding that does not go through the window's own queue, as
  cloaking does not, is never posted. A change applied at once, a move or a
  resize alike, would be overtaken by a frame still queued. So callers never
  wait for a frame before placing, hiding or showing a window: a motion ends
  with `place` and `repaint` at its target, which land after its frames. A
  caller stops sending a window frames before hiding or minimizing it, since
  a frame posted after lands after the change. Past the grace, any move of
  the window, whoever made it, ends the order, and a change made at once
  then lands before anything still queued: the later of two frames queued,
  when the application took the earlier and stopped before it was reported
  not responding, or what was queued when something else moved the window.
- `FrameControl::posts_behind` tells the caller whether a change made to a
  window now is posted. Asked right before the change, its answer can only go
  from posted to made at once by the time the change is made, as time passes,
  or once the window is seen to move or its application responds again, and
  the other way once its application stops responding, or once a frame is
  given to it while its application does not respond, which posts nothing
  but starts what the table keeps for it. A caller that keeps its own record
  of how a change was made asks right after it: a posted change renews the
  grace, so `true` then means it was posted, or that the application stopped
  responding since, which is handled the same way. A posted change
  completes when the application gets to it: until then the desktop may not
  show it, so the caller must not read it back as done, and a window may read
  as settled while its last placement is still queued. It does not wait for
  the application, except where a platform cannot end the maximized state of
  a window out of view without waiting: then as long as the application takes
  to handle that, once the window is out of view, and unless the application
  is already reported not responding. Otherwise the window stays maximized,
  the exception above.
  A posted placement ends a maximized state as one made at once does, that of
  a window maximized when it is posted, and that of one minimized then which a
  show queued ahead brings back maximized. It lands with the bounds the window
  has in its normal state, even on one still minimized or maximized when it
  was posted. A posted repaint only repairs what was wrong before the changes
  queued ahead of it, so the caller repaints the window again once it reports
  where a posted placement took it. Focus is never posted: given right after a
  posted show, it may reach the window before the window is shown.
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
of the window. A step that answers the pointer, such as a click on the bar,
asks for no pointer move at all (see [actions.md](actions.md#origins)). Borders and window corners are not window changes: they go
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
| `place` | `set_rect()`: clears `WS_MAXIMIZE`, then `SetWindowPos` with frame compensation (see below); while something posted to the window may be queued, `post_placement()`: the same move posted with `SWP_ASYNCWINDOWPOS`, behind it, after a restore posted for a maximized window in view, or a minimized one a show queued ahead brings back in view (see [Frames of a Transition](#frames-of-a-transition)) |
| `pre_place` | `move_async()`: the same move with `SWP_ASYNCWINDOWPOS`, which posts the request instead of sending it |
| `place_frames` (`FrameControl`) | The core `FrameTable` decides, reading windows through `IsWindow`, `IsIconic`, `IsZoomed`, `IsHungAppWindow`, `GetWindowRect` and `GetWindowThreadProcessId`, and with the first frame of a motion the invisible borders (`DwmGetWindowAttribute` with `DWMWA_EXTENDED_FRAME_BOUNDS`), the DPI of the display they are drawn at (`GetDpiForMonitor`), and whether the window follows the display's scale (the DPI awareness contexts of the window and of the thread); every frame of a window that does not follow it also reads its scale and its display's (`GetDpiForWindow`, `GetDpiForMonitor`); a frame is posted with `SWP_ASYNCWINDOWPOS` (see [Frames of a Transition](#frames-of-a-transition)) |
| `repaint` | `redraw()`: `RedrawWindow` with aggressive flags, then realigns a Chromium window's GPU surface; without `RDW_UPDATENOW` while something posted to the window may be queued, so it paints after it |
| `focus` | `SetForegroundWindow`, never posted |
| `put_behind` | `SetWindowPos` with `other` as the window to insert after and `SWP_NOMOVE`, `SWP_NOSIZE` and `SWP_NOACTIVATE`, only when both or neither are `WS_EX_TOPMOST`: inserted after a window of the other band, a window moves into that band; with `SWP_ASYNCWINDOWPOS` too while something posted to the window may be queued, behind it |
| `close` | `PostMessageW` with `WM_CLOSE` |
| `minimize` | `ShowWindow(SW_MINIMIZE)`, or `ShowWindowAsync(SW_MINIMIZE)` while something posted to the window may be queued, behind it |
| `hide` | `Cloak`: ImmersiveShell cloak, never posted; `Hide`: `ShowWindow(SW_HIDE)`, or `ShowWindowAsync` as for `minimize`; `Minimize`: as `minimize` |
| `show` | `Cloak`: uncloak, never posted; otherwise `ShowWindow(SW_SHOWNOACTIVATE)`, or `ShowWindowAsync` as for `minimize` |
| `hiding_events` | Every way. Cloaking fires no hide or minimize event; `SW_HIDE` fires `EVENT_OBJECT_HIDE`; `SW_MINIMIZE` fires `EVENT_SYSTEM_MINIMIZESTART`, and showing the window again `EVENT_SYSTEM_MINIMIZEEND` |
| `reveal` | uncloak, then `ShowWindow(SW_SHOW)`, or `ShowWindowAsync` as for `minimize` |
| `move_cursor` | `SetCursorPos` |

`ShowWindow` reports whether the window was visible before, not whether the
call worked, so `minimize`, `hide`, `show` and `reveal` first check with
`IsWindow` that the window still exists. `ShowWindowAsync` does say whether it
queued the change, and a failure comes back as an error, as does a cloak or
uncloak that fails, for a window the shell has no view of for example.

### Whether Windows Animate

`Desktop::animations_allowed` says whether windows may be animated, and is
read again each time it is asked, so a change of the setting applies to the
next transition. A platform without such a setting answers `true`, the
default. `Win32Desktop` answers from `system_settings.rs`:
`SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION)`, the setting behind both
the Windows 11 "Animation effects" and the Windows 10 "Show animations in
Windows" toggles, and `false` in a remote session
(`GetSystemMetrics(SM_REMOTESESSION)`), where every frame would travel over
the network. A setting that cannot be read allows animations.

## Window Positioning

`set_rect()` uses `SetWindowPos` with specific flags:

- `SWP_NOZORDER` -- do not change Z-order
- `SWP_NOACTIVATE` -- do not activate the window
- `SWP_NOSENDCHANGING` -- suppress `WM_WINDOWPOSCHANGING` so windows cannot
  reject the resize
- `SWP_NOCOPYBITS` -- do not copy client area bits (prevents visual artifacts)
- `SWP_FRAMECHANGED` -- conditionally added for Chromium/Firefox windows

### Frames of a Transition

`place_frames` (`window_control/frames.rs` and `frames_post.rs`) posts each
frame with `SetWindowPos` and these flags:

- `SWP_ASYNCWINDOWPOS` -- post the move to the window's thread instead of
  sending it, so a frame never waits for the application
- `SWP_NOZORDER`, `SWP_NOACTIVATE` -- leave the stacking order and focus alone
- `SWP_NOSENDCHANGING` -- as `set_rect()`, so the minimum size an application
  asks for clamps no frame that the final placement would not

`SWP_NOCOPYBITS` is never used: it would discard the client area and make the
application repaint all of it on every frame of a plain move. Resize frames
do not add `SWP_FRAMECHANGED` for Chromium windows either: Windows sends
`WM_NCCALCSIZE` whenever the size changes, as it does while the user resizes a
window, and the final `place` still adds it and `repaint` realigns the GPU
surface.

`Win32WindowControl` keeps a core `FrameTable`, which makes every decision, and
reads and moves windows for it through a `FrameProbe` (`Probe` in
`window_control/frames.rs`), on `Instant::now()`. Where a window is comes from
`GetWindowRect`, in outer bounds, and the table compares it with where the
window was when its last frame was posted: a frame posted less than
`PENDING_LIMIT` ago to a window not seen anywhere else since holds the next
one back. A window whose application is not responding (`IsHungAppWindow`)
gets nothing posted. With the first frame of a motion, the probe reads the
window's invisible borders, the DPI of its display they are drawn at, and
whether Windows maps the moves made to it to pixels of its own. On every
lookup it reads the window's owner, the thread and
process `GetWindowThreadProcessId` gives, so a new window given the handle of
an old one is told apart. When a frame lands, Windows reports
`EVENT_OBJECT_LOCATIONCHANGE`, which arrives as `LocationChanged`. No frame is
logged.

The daemon is per-monitor DPI aware. A window that is not has pixels of its
own, and on a display scaled up Windows maps every move made to it to them and
back, rounding its position and its size each on their own, halves away from
zero, as `MulDiv` does. A move of less than one of those pixels leaves the
window where it is. So the probe maps such a window's frames the same way
(`window_control/frames_dpi.rs`), with the window's DPI (`GetDpiForWindow`)
and its display's (`GetDpiForMonitor`), and a frame that maps to where the
window is is not posted. `PhysicalToLogicalPointForPerMonitorDPI` maps only
points within the window, so it does not do here. The mapping has been
checked on displays at 125% and 200% (120 and 192 DPI); 150 and 175%, a
monitor whose origin is not a multiple of its scale, and monitors at
different scales have not been tried. A miss holds the next frame back for
one limit at most.

A thread handles what is posted to it in order, so a window's other changes
keep their order with its frames by being posted too
(`window_control/ordered.rs`). While something posted to a window may still
be queued, as the table's `posts_behind` says, and
`FrameControl::posts_behind` tells the caller:

- `place` posts its move with `SWP_ASYNCWINDOWPOS`, with the flags and the
  Chromium rule of `set_rect()` (`post_placement()` in
  `window_control/posted_placement.rs`, whose decisions are plain functions
  tested on their own). A move made at once, a resize as well as a plain move,
  would be overtaken by a frame still queued. A minimized or maximized window
  is moved with the invisible borders read with its frames, in its normal
  state, which it is in once the move lands after a restore
  (`window_control/frames_kept.rs`). Once its display's DPI is no longer the
  one they were read at, a window Windows stretches has them scaled by the
  ratio, and any other, whose frame Windows draws at its own DPI, has each
  side moved by the change in `SM_CXSIZEFRAME` and `SM_CXPADDEDBORDER`
  (`SM_CYSIZEFRAME` at the bottom) from `GetSystemMetricsForDpi`. That holds
  for a window per-monitor aware of the second version, and is assumed for
  one of the first, whose frame Windows scales only when its application asks
  it to. A maximized window is restored first, with
  `ShowWindowAsync(SW_SHOWNOACTIVATE)` posted ahead of the move, when it is to
  be in view once what was posted lands: as the last change of its show state
  posted says, until the window is seen as that change leaves it after being
  seen otherwise, or as its own `WS_VISIBLE` does, which a cloaked window
  keeps. So is any minimized window a show queued ahead brings back in view:
  behind that show, the restore ends the maximized state it may come back in,
  and does nothing to one it brings back in its normal state. A window out of
  view would be shown by a restore, so it has `WS_MAXIMIZE` cleared at once
  instead, as `set_rect()` does, which waits as long as the application takes
  to handle the change of style, but only once the window no longer has
  `WS_VISIBLE`. While the hide or minimize that takes it out of view is still
  queued, that change could land between the read of the style and its
  write, which would undo it, so the clear is skipped, as it is while
  `IsHungAppWindow` says the application does not respond. The window then
  stays maximized, and a later show brings it back as if it had not been
  placed: a hidden one to the normal bounds it had before it was maximized,
  where `rcNormalPosition` still has it.
- `repaint` invalidates without `RDW_UPDATENOW`, which would wait for the
  application to paint; it paints once it gets to its messages, after the
  move. A Chromium window's GPU surface is still realigned: that never waits,
  as it reads where the surface and the client area are now and posts the
  surface's move, but it only repairs a drift that is there before what was
  posted lands.
- `put_behind` posts its change of the stacking order with
  `SWP_ASYNCWINDOWPOS` too.
- `minimize`, `hide` and `show` without cloaking, and `reveal` use
  `ShowWindowAsync`: a minimize made at once would have a frame landing after
  it move the minimized window's caption on screen, and a show made at once
  ahead of a minimize still queued would leave the window minimized. Cloaking
  goes through the shell, outside the window's queue, and is never posted.
- `focus` is never posted: given right after a posted show, it may reach the
  window before the window is shown.

Each posted change but a repaint, which is never noted, is noted with the
table, with where the window was, so the ones after it are posted behind it
too, and a change of show state with whether it leaves the window in view,
made at once or not; a reveal of a minimized window leaves it out of view, as
`SW_SHOW` keeps it minimized. A change that fails is not noted. Once
transitions are off, `settle_responding` forgets the windows past the grace
whose application responds. A window never given a frame gets every change
at once, as it always did, whether its application responds or not; one that
was gets its changes posted while its application does not respond, as a
change made at once would wait for it.

### How the Tiler Uses Frames

The tiling manager keeps every moving window in `motion::Motions`, and takes a
frame step on every display frame while windows move (see
[tiling-layout.md](tiling-layout.md#placements-that-animate)):

- It posts every frame due in one `place_frames` call, and moves the border
  of each moving window that keeps its size to where the outcome found the
  window, or to the frame now posted when the window took the one before
  exactly; one that changes size has its border hidden until it is in place.
- A motion ends at its end time with `place` and `repaint` at its exact
  target, even when the window is there already. It does not wait for its
  last frames to land: the platform orders the placement behind them.
- A window found `Pending` and not seen to move for 50 ms (`MAX_LAG`), since
  its motion started or since it handled its first frame, ends every motion
  of its step at once, through the same `place`. One whose application
  stopped responding after it was given frames needs nothing more: its
  placement is posted behind them. One that had stopped before any was gets
  nothing posted, its placement included, which is made at once, as it is
  without transitions, and waits for it.
- A window found `Maximized`, `Minimized` or `Gone` is left alone: its motion
  stops where the window is, and nothing forces it back.
- Hiding or minimizing a moving window stops its frames first: a frame
  posted after the change would land after it.
- Only the first frame of a motion is marked `first`. A new target mid-flight
  goes on with the same motion, so the platform keeps what it knows of the
  frame on its way.
- `Unsupported` turns transitions off for the rest of the session: the
  windows in flight are placed at their targets at once, with their borders,
  and every later placement snaps. Any other error places the windows the
  frames were for at their targets, and transitions stay on.
- While `posts_behind` holds for a window, the tiler does not read what it
  sent back as done (`applier_posted.rs`), and it asks right after each
  change it keeps a record of. A placement is not skipped because the
  desktop has the window in place already, unless the window was last sent
  there too, by a placement or a frame. `Focus { move_cursor: true }` and
  `CenterCursor` aim at the center of where the window was last sent, and a
  new motion starts from there. A posted placement that finds the window
  there already, within 4 pixels, where the last thing sent took it, to
  the pixel, moves nothing and reports nothing, so the window has its
  border drawn at once. Any other is repainted again, and has its border
  drawn, once the window reports being within 4 pixels of it, as a posted
  repaint asks, or 500 ms after it if the window has moved since or is
  there; one that has not moved lands on its next report. A posted hide,
  minimize or show that changes something keeps the window as it leaves it
  in the tiler's view of the desktop until it lands. A window placed while
  it is out of view, or is to be, with its changes posted, counts as
  maximized as long as the desktop says so, and a display change shows
  such a window, if it is hidden, before it places it.
- Once transitions are off, by the config, by the system's setting, or as
  the platform has no frames, the tiler calls `settle_responding` as they
  turn off and at the start of every step after, before the tiler reads
  the desktop, and of every flush, for as long as a window it gave frames
  to is still posted to.

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

# Window Event System

Mosaico reacts to what happens to windows and displays through a
platform-agnostic event model. Core defines the events and the contract a
platform follows to produce them. The Windows platform implements it with
WinEvent hooks and a hidden window that receives broadcasts.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/event/window_event.rs` | `WindowEvent` and what each variant promises |
| `crates/mosaico-core/src/event/drag.rs` | `DragStart`, `Grab` and `Edges` -- what the start of a drag carries |
| `crates/mosaico-core/src/event/source.rs` | `EventSource` contract and `EventWatch` |
| `crates/mosaico-core/src/event/hover.rs` | `MousePointer` contract and `FocusFollowsMouse` |
| `crates/mosaico-core/src/event/rest.rs` | `HoverTracker` -- when the pointer has come to rest |
| `crates/mosaico-windows/src/event_source.rs` | `WindowsEventSource` and its Win32 notes |
| `crates/mosaico-windows/src/event_source_hooks.rs` | The two `SetWinEventHook` hooks and their callback |
| `crates/mosaico-windows/src/event_source_sink.rs` | The hidden window for `WM_DISPLAYCHANGE` and `WM_SETTINGCHANGE` |
| `crates/mosaico-windows/src/event.rs` | `translate()` and `translate_broadcast()` -- Win32 values to `WindowEvent` |
| `crates/mosaico-windows/src/drag_facts.rs` | What the start and the end of a drag carry, read in the hook |
| `crates/mosaico-windows/src/pointer.rs` | `WindowsPointer` -- the window under the pointer |
| `crates/mosaico-windows/src/message_thread.rs` | `MessageThread` -- a thread with a message queue |

## WindowEvent Variants

| Variant | Reported when |
|---------|---------------|
| `Created { window }` | A window is created (before it is first shown, where the platform can tell) or shown |
| `Destroyed { window }` | A window is closed and no longer exists |
| `Hidden { window }` | A window is hidden, but not necessarily closed: it stops being shown without being minimized, including a window that closes while shown (just before `Destroyed`). May be reported for a window that stays on screen |
| `Focused { window }` | A window becomes the active one, including when the tiler focused it |
| `MouseHover { window }` | The pointer came to rest on a window (focus-follows-mouse only) |
| `MoveStarted { window, start }` | The user starts dragging a window to a new place or size; `Moved` follows when the drag ends. `start` says what the pointer holds the window by, where the window is, whether it is maximized and where the pointer is, as the platform saw the drag start |
| `Moved { window, pointer }` | The user finishes dragging a window to a new place or size, with where the pointer is as the platform saw the drag end |
| `Minimized { window }` | A window is minimized |
| `Restored { window }` | A minimized window is shown again |
| `TitleChanged { window }` | A window's title changes |
| `LocationChanged { window }` | A window's position or size changes, for any reason |
| `DisplayChanged` | Monitors are connected, disconnected, rearranged or change resolution |
| `WorkAreaChanged` | A taskbar, dock or panel changes the usable area of a monitor |

### Methods

- `window()` -- the window the event is about, `None` for display events
- `name()` -- returns the event name as a `&'static str`
- `Display` implementation -- formats as `"EventName 0x1A0B2C"`, or just the
  name for display events

## The EventSource Contract

`EventSource::start(sender)` returns an `EventWatch` once the source is
watching. Stopping or dropping the watch stops the source, and once it has
stopped nothing more is sent and the sender is dropped. Every platform
follows these rules:

- Mosaico's own windows (bars, borders) are never reported. What mosaico
  does to other windows is reported like anything else.
- Events may name windows that will never be managed, and any event may be
  repeated: each one is a reason to look at the window again.
- Events about one window keep their order. Nothing else is promised across
  windows, except that focus changes keep their order whichever windows they
  name, since the tiler follows the last one.
- A kind of event the platform cannot observe is left out, never made up.
- `start` fails, and sends nothing, when the platform cannot watch windows.
  When it can watch windows but not the displays, `start` still succeeds,
  and `EventWatch::display_error` says why display and work area changes
  will not come.

The details of each variant are in the docs of `WindowEvent`.

## Win32 Event Translation

`translate()` maps the values the WinEvent hooks receive, and
`translate_broadcast()` the messages the hidden window receives:

| Win32 | WindowEvent |
|-------|-------------|
| `EVENT_OBJECT_CREATE`, `EVENT_OBJECT_SHOW` | `Created` |
| `EVENT_OBJECT_DESTROY` | `Destroyed` |
| `EVENT_OBJECT_HIDE` | `Hidden` |
| `EVENT_SYSTEM_FOREGROUND` | `Focused` |
| `EVENT_SYSTEM_MOVESIZESTART` | `MoveStarted` |
| `EVENT_SYSTEM_MOVESIZEEND` | `Moved` |
| `EVENT_SYSTEM_MINIMIZESTART` | `Minimized` |
| `EVENT_SYSTEM_MINIMIZEEND` | `Restored` |
| `EVENT_OBJECT_NAMECHANGE` | `TitleChanged` |
| `EVENT_OBJECT_LOCATIONCHANGE` | `LocationChanged` |
| `WM_DISPLAYCHANGE` | `DisplayChanged` |
| `WM_SETTINGCHANGE` with `SPI_SETWORKAREA` | `WorkAreaChanged` |

### Filtering

- Reports that name no window, such as the mouse pointer being shown or
  changing shape, are dropped.
- Reports on child objects (`id_object != OBJID_WINDOW`: scroll bars,
  carets, client areas) are dropped, except creation, show and name changes.
  WPF windows such as Visual Studio's report those only on an object inside
  the window.
- Setting changes other than the work area are dropped.
- Unrecognized event codes return `None` (silently ignored).

Windows hides a window as it destroys it, so a window that closes while
shown reports `EVENT_OBJECT_HIDE` and then `EVENT_OBJECT_DESTROY`: `Hidden`,
then `Destroyed`.

Both functions are pure. `event_tests.rs` and, for the start of a drag,
`event_drag_tests.rs` check them with the values Windows reports in each of
these cases, including the hide and show a GPU-accelerated application
reports while it recreates its drawing surface.

### What a Drag Carries

`translate()` gives the start and the end of a drag without what was read
with them. The hook's callback reads it as it reports each
(`drag_facts::read`), since by the time the tiler handles the event the
window and the pointer have moved on:

- the pointer, with `GetCursorPos`, at the start and at the end;
- at the start, the window's visible bounds, as the desktop reads them,
  whether it is maximized (`IsZoomed`), which a drag by the title bar soon
  ends, and what the window answers to `WM_NCHITTEST` at the pointer:
  `HTCAPTION` is `Grab::TitleBar`, and each sizing edge or corner of the
  frame is `Grab::Edges` with that edge, or those two. Anything else, such
  as the client area of a window whose application starts the drag itself,
  leaves `grab` empty. The window moves or resizes with the pointer, so
  what is under it is what it was grabbed by, even when the hook reports a
  little late.

The point is mapped to the window's own coordinates first
(`PhysicalToLogicalPointForPerMonitorDPI`), for a window that is not aware
of the display's DPI. The window is asked with `SendMessageTimeoutW`, with
`SMTO_BLOCK`, so the event thread handles nothing else meanwhile and no
report overtakes this one, and `SMTO_ABORTIFHUNG`, for at most 50 ms; a
window that does not answer holds nothing. `drag_facts_tests.rs` checks the
codes, `drag_facts_read_tests.rs` a start and an end read on a hidden window
of the test's own, and `drag_facts_hit_tests.rs` what a hidden window with
a caption and a sizing frame answers: one that is per-monitor aware, at its
own pixels on any display, and one that is not DPI aware, asked from a
per-monitor aware thread at points in the display's pixels, on a display
that is scaled.

## Event Hook Configuration

`WindowsEventSource::start()` sets up, on a thread of its own:

- Two `SetWinEventHook` hooks, for `0x0003` to `0x0017` (system events) and
  `0x8000` to `0x800C` (object events), instead of one range that would also
  deliver the thousands of unrelated events in between
- Flags:
  - `WINEVENT_OUTOFCONTEXT` -- the callback runs in Mosaico's process, on the
    thread that set the hook, while that thread pumps messages. Such reports
    arrive in the order they were made, and both hooks deliver on the one
    thread, so every event keeps its order, focus changes included.
  - `WINEVENT_SKIPOWNPROCESS` -- ignores events from Mosaico's own windows
    (bars, borders)
- A hidden `WS_EX_TOOLWINDOW` window that receives `WM_DISPLAYCHANGE` and
  `WM_SETTINGCHANGE`. A message-only window would not get broadcasts.

If the hooks cannot be installed, `start` returns the error and removes
whichever hook was installed. If only the hidden window cannot be created,
the source runs without display and work area changes, and
`EventWatch::display_error` says why: the daemon logs it, and
`mosaico debug events` prints it as a warning. The window's class is
registered once per process, so a class of that name that something else
registered counts as a failure.

## Event Flow

```
Window state change in OS
  -> WinEvent hook callback (win_event_proc) on the event thread
  -> event::translate(event_code, hwnd, id_object)
  -> WindowEvent (or None if filtered/unrecognized)
  -> the mpsc::Sender<WindowEvent> given to EventSource::start
  -> bridge thread wraps as DaemonMsg::Event
  -> TilingManager::handle_event()
```

## Focus-Follows-Mouse

`MouseHover` does not come from the event source. While `focus_follows_mouse`
is on, `FocusFollowsMouse` polls a `MousePointer` every 100 ms on a thread of
its own, and reports a window once the pointer has stayed on it for 3 polls
in a row, and only once until the pointer moves to another window. A poll
that finds no window is skipped. Turning the setting off in `config.toml`
stops the thread, and turning it back on starts a new one. On Windows the
pointer is read with `GetCursorPos`, `WindowFromPoint` and
`GetAncestor(GA_ROOT)`. While the bar takes the pointer, a pointer resting on
what it draws finds the bar, which the tiler does not manage, so it focuses
nothing; a bar that lets the pointer through, over a maximized window or with
the bar's `[mouse] enabled = false`, finds the window under it, as before.

Focus-follows-mouse rests while the user drags any window, tiled or not: the
tiling manager drops each report until the window is let go
(`applier_reports.rs`), as a focus change then would reach the system's move
or size loop. The next report once the window is let go applies as usual.

## How Events Drive Tiling

| Event | Tiling Response |
|-------|-----------------|
| `Created` | Place a window that is not shown yet; adopt a visible one into the focused monitor's workspace |
| `Destroyed` | Remove window from workspace, re-tile |
| `Hidden` | Remove the window, if it is really no longer visible, and stop its transition then |
| `Focused` | Update focus tracking, show/reposition border; another window taking focus, other than by the tiler's doing, ends a drag |
| `MouseHover` | Focus the window, if it is managed and on the active workspace; dropped while windows move and while the user drags a window, and not repeated until the pointer moves to another window. That rarely matters: a report takes 3 polls, at least 200 ms, longer than a default 180 ms transition, so a window sliding under a pointer at rest is usually reported only once it stops |
| `MoveStarted` | Stop the window's transition, if it is moving, drop what is held back for it, and leave the pointer alone until the drag ends. A tiled window keeps its slot, but nothing places it until the drag ends, whatever the user does with the keyboard or the bars meanwhile, and its border is hidden once the window has left where it started; the layout changes then, with `Moved` |
| `Moved` | Drop the window whose drag the tiler followed where the pointer is, with `[mouse] drag_to_swap`; otherwise detect monitor reassignment of a window on a workspace on screen, re-tile if needed (see [tiling-layout.md](tiling-layout.md#dragging-windows)) |
| `Minimized` | Remove from workspace, re-tile, and stop the window's transition, if it is really minimized |
| `Restored` | Re-add to workspace, re-tile |
| `TitleChanged` | Adopt a window that only became recognizable once it had a title |
| `LocationChanged` | Notice any window on screen starting or stopping to be maximized or fullscreen, for the bars (`bar_filling.rs`), the one the user drags too; then hide the border of a window the user drags once it has left where it started, and move the preview of where it would land, or otherwise notice the focused window entering or leaving maximized state; once a window still to land where a posted placement takes it reports being there, or being anywhere once the wait for it by time is over, repaint it again and draw its border there (`applier_landing.rs`) |

A drag of a tiled window lasts from its `MoveStarted` to its `Moved`.
Windows does not say that `EVENT_SYSTEM_MOVESIZEEND` follows every
`EVENT_SYSTEM_MOVESIZESTART`, only that an out-of-context hook gets its
events in the order they happen. So a drag also ends when its window
closes, hides, is minimized or leaves the workspace on screen, when
another window takes focus other than by the tiler's doing, and when the
user drags another window. A drag whose end is lost while the user goes
on with the keyboard alone keeps its window out of the layout, with its
border hidden, until one of those happens.

One timing question is open until it is checked on a live desktop:
whether a snap or a maximize the system makes at a drop has landed by the
time the daemon reads the window's rect for its `Moved`. If it lands
after, a drop at an edge of the monitor is taken for a move onto the tile
under the pointer, a swap or a join, and the snap then lands over the new
layout.

## Design Decisions

- **Platform-agnostic event enum and contract** in `mosaico-core` keep the
  tiling logic free of Win32 dependencies. A future Linux implementation
  would implement `EventSource` and `MousePointer` with X11 or Wayland events.
- **`OBJID_WINDOW` filter** is essential -- without it, every scrollbar
  scroll, button highlight, and caret blink would trigger event processing.
- **`WINEVENT_SKIPOWNPROCESS`** prevents the border overlay window from
  creating recursive event loops (border created -> `Created` event -> tiling
  tries to manage border -> ...).
- **The layout reacts to `EVENT_SYSTEM_MOVESIZEEND`** (`Moved`), not to
  `EVENT_SYSTEM_MOVESIZESTART` (`MoveStarted`) or the position reports in
  between, so it retiles once after the user finishes dragging a window,
  not throughout the drag.
- **`EVENT_OBJECT_HIDE` is `Hidden`, not `Destroyed`**, because
  GPU-accelerated applications report a hide while they recreate their
  surface and stay on screen. The tiler checks visibility before it removes
  the window.
- **Window events, hotkeys and focus-follows-mouse run on separate
  threads**, so each can be implemented on its own, and a flood of window
  events cannot hold a hotkey back.

# Window Event System

Mosaico reacts to what happens to windows and displays through a
platform-agnostic event model. Core defines the events and the contract a
platform follows to produce them. The Windows platform implements it with
WinEvent hooks and a hidden window that receives broadcasts.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/event.rs` | `WindowEvent` and what each variant promises |
| `crates/mosaico-core/src/event/source.rs` | `EventSource` contract and `EventWatch` |
| `crates/mosaico-core/src/event/hover.rs` | `MousePointer` contract and `FocusFollowsMouse` |
| `crates/mosaico-core/src/event/rest.rs` | `HoverTracker` -- when the pointer has come to rest |
| `crates/mosaico-windows/src/event_source.rs` | `WindowsEventSource` and its Win32 notes |
| `crates/mosaico-windows/src/event_source_hooks.rs` | The two `SetWinEventHook` hooks and their callback |
| `crates/mosaico-windows/src/event_source_sink.rs` | The hidden window for `WM_DISPLAYCHANGE` and `WM_SETTINGCHANGE` |
| `crates/mosaico-windows/src/event.rs` | `translate()` and `translate_broadcast()` -- Win32 values to `WindowEvent` |
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
| `MoveStarted { window }` | The user starts dragging a window to a new place or size; `Moved` follows when the drag ends |
| `Moved { window }` | The user finishes dragging a window to a new place or size |
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

Both functions are pure. `event_tests.rs` checks them with the values Windows
reports in each of these cases, including the hide and show a
GPU-accelerated application reports while it recreates its drawing surface.

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
`GetAncestor(GA_ROOT)`.

## How Events Drive Tiling

| Event | Tiling Response |
|-------|-----------------|
| `Created` | Place a window that is not shown yet; adopt a visible one into the focused monitor's workspace |
| `Destroyed` | Remove window from workspace, re-tile |
| `Hidden` | Remove the window, if it is really no longer visible, and stop its transition then |
| `Focused` | Update focus tracking, show/reposition border |
| `MouseHover` | Focus the window, if it is managed and on the active workspace; dropped while windows move. That rarely matters: a report takes 3 polls, at least 200 ms, longer than a default 180 ms transition, so a window sliding under a pointer at rest is usually reported only once it stops |
| `MoveStarted` | Stop the window's transition, if it is moving; the layout changes when the drag ends, with `Moved` |
| `Moved` | Detect monitor reassignment, re-tile if needed |
| `Minimized` | Remove from workspace, re-tile, and stop the window's transition, if it is really minimized |
| `Restored` | Re-add to workspace, re-tile |
| `TitleChanged` | Adopt a window that only became recognizable once it had a title |
| `LocationChanged` | Notice the focused window entering or leaving maximized state |

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

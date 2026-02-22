# Window Event System

Mosaico uses a platform-agnostic event model that represents meaningful OS
window state changes. The Windows platform translates raw Win32 `WinEvent`
codes into these events.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/event.rs` | `WindowEvent` enum definition |
| `crates/mosaico-windows/src/event.rs` | `translate()` -- maps Win32 events to `WindowEvent` |

### Key Types

- `WindowEvent` (enum) -- platform-agnostic window state change events
- `translate(event, hwnd, id_object)` -- Win32 event translator function

## WindowEvent Variants

| Variant | Description |
|---------|-------------|
| `Created { hwnd }` | A new window appeared on the desktop |
| `Destroyed { hwnd }` | A window was closed or hidden |
| `Focused { hwnd }` | A window received keyboard focus |
| `Moved { hwnd }` | A window finished a move or resize operation |
| `Minimized { hwnd }` | A window was minimized |
| `Restored { hwnd }` | A window was restored from minimized state |
| `TitleChanged { hwnd }` | A window's title text changed |

### Methods

- `hwnd()` -- extracts the handle from any variant
- `name()` -- returns the event name as a `&'static str`
- `Display` implementation -- formats as `"EventName 0x{HWND:X}"`

## Win32 Event Translation

`translate()` maps raw Win32 accessibility events to `WindowEvent`:

| Win32 Constant | WindowEvent |
|----------------|-------------|
| `EVENT_OBJECT_SHOW` | `Created` |
| `EVENT_OBJECT_CREATE` | `Created` |
| `EVENT_OBJECT_DESTROY` | `Destroyed` |
| `EVENT_OBJECT_HIDE` | `Destroyed` |
| `EVENT_SYSTEM_FOREGROUND` | `Focused` |
| `EVENT_OBJECT_FOCUS` | `Focused` |
| `EVENT_SYSTEM_MOVESIZEEND` | `Moved` |
| `EVENT_SYSTEM_MINIMIZESTART` | `Minimized` |
| `EVENT_SYSTEM_MINIMIZEEND` | `Restored` |
| `EVENT_OBJECT_NAMECHANGE` | `TitleChanged` |

### Filtering

- Events with `id_object != OBJID_WINDOW` (constant 0) are filtered out.
  This excludes events on child objects like scrollbars, buttons, and other
  non-window UI elements.
- Unrecognized event codes return `None` (silently ignored).

## Event Hook Configuration

The Win32 event hook is set up in `event_loop::start()`:

- API: `SetWinEventHook`
- Event range: `EVENT_MIN=0x0003` to `EVENT_MAX=0x800C`
- Flags:
  - `WINEVENT_OUTOFCONTEXT` -- callback runs in Mosaico's process, not the
    target window's process
  - `WINEVENT_SKIPOWNPROCESS` -- ignores events from Mosaico's own windows
    (border overlay, etc.)

## Event Flow

```
Window state change in OS
  -> SetWinEventHook callback (win_event_proc)
  -> event::translate(event_code, hwnd, id_object)
  -> WindowEvent (or None if filtered/unrecognized)
  -> thread-local EVENT_SENDER (mpsc::Sender<WindowEvent>)
  -> bridge thread wraps as DaemonMsg::Event
  -> TilingManager::handle_event()
```

## How Events Drive Tiling

The `TilingManager` reacts to each event type:

| Event | Tiling Response |
|-------|-----------------|
| `Created` | Add window to focused monitor's workspace, re-tile |
| `Destroyed` | Remove window from workspace, re-tile |
| `Focused` | Update focus tracking, show/reposition border |
| `Moved` | Detect monitor reassignment, re-tile if needed |
| `Minimized` | Remove from workspace, re-tile |
| `Restored` | Re-add to workspace, re-tile |
| `TitleChanged` | (Currently no tiling action) |

## Design Decisions

- **Platform-agnostic event enum** in `mosaico-core` keeps the tiling logic
  free of Win32 dependencies. A future Linux implementation would translate
  X11/Wayland events to the same `WindowEvent` variants.
- **`OBJID_WINDOW` filter** is essential -- without it, every scrollbar
  scroll, button highlight, and caret blink would trigger event processing.
- **`WINEVENT_SKIPOWNPROCESS`** prevents the border overlay window from
  creating recursive event loops (border created -> `Created` event -> tiling
  tries to manage border -> ...).
- **`EVENT_SYSTEM_MOVESIZEEND`** (not `MOVESTART` or continuous move events)
  ensures we only react once after the user finishes dragging a window,
  avoiding constant re-tiling during the drag operation.
- **Both `SHOW`/`CREATE` and `DESTROY`/`HIDE`** are mapped to `Created` and
  `Destroyed` because different applications use different event codes for
  window lifecycle. Covering both ensures reliable detection.

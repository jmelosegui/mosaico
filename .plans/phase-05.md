# Phase 5: Event-Driven Window Discovery

**Status:** Pending

**Goal:** Hook into Windows event system so the daemon can react to windows being
opened, closed, moved, focused, minimized, and restored in real time.

## Why events instead of polling?

Polling (repeatedly calling `EnumWindows`) is wasteful and slow. Windows provides
`SetWinEventHook` — a callback-based API that notifies your process whenever a
window event occurs. This is how all serious window managers work.

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      event.rs                # Platform-agnostic WindowEvent enum
  mosaico-windows/
    src/
      event.rs                # Map raw Win32 events to WindowEvent
      event_loop.rs           # SetWinEventHook + message pump (GetMessageW)
      daemon.rs               # Updated to start the event loop
```

## Tasks

- [ ] Define `WindowEvent` enum in `mosaico-core` (Created, Destroyed, Focused,
      Moved, Minimized, Restored, TitleChanged)
- [ ] Implement `SetWinEventHook` wrapper in `mosaico-windows`
- [ ] Implement Win32 message pump (`GetMessageW` / `DispatchMessageW`)
- [ ] Map raw Win32 event codes to `WindowEvent` variants
- [ ] Integrate event loop into the daemon main loop
- [ ] Log events to stdout/stderr for debugging
- [ ] Add `mosaico debug events` subcommand to watch events in real time
- [ ] Commit

## Win32 APIs used

| Function | Purpose |
|----------|---------|
| `SetWinEventHook` | Register a callback for window events |
| `UnhookWinEvent` | Unregister the event hook |
| `GetMessageW` | Block until a message is available (message pump) |
| `TranslateMessage` | Translate virtual-key messages |
| `DispatchMessageW` | Dispatch a message to the window procedure |

## Key Win32 events

| Event | Meaning |
|-------|---------|
| `EVENT_OBJECT_CREATE` | A window was created |
| `EVENT_OBJECT_DESTROY` | A window was destroyed |
| `EVENT_OBJECT_SHOW` | A window became visible |
| `EVENT_OBJECT_HIDE` | A window was hidden |
| `EVENT_OBJECT_FOCUS` | A window received focus |
| `EVENT_SYSTEM_FOREGROUND` | The foreground window changed |
| `EVENT_SYSTEM_MOVESIZESTART` | A window started being moved/resized |
| `EVENT_SYSTEM_MOVESIZEEND` | A window finished being moved/resized |
| `EVENT_SYSTEM_MINIMIZESTART` | A window was minimized |
| `EVENT_SYSTEM_MINIMIZEEND` | A window was restored from minimized |
| `EVENT_OBJECT_NAMECHANGE` | A window title changed |

## Rust learning notes

- **Channels (`std::sync::mpsc`)** — multi-producer single-consumer channels for
  sending events between threads. The event hook callback runs on a dedicated
  thread with a message pump; events are sent over a channel to the main daemon
  thread. Similar to `Channel<T>` in C#.
- **Threading (`std::thread`)** — spawn OS threads for the event loop and message
  pump. The daemon needs at least two threads: one for the Win32 message pump
  and one for processing events.
- **`extern "system"` callbacks** — same pattern as `EnumWindows` from Phase 3b,
  but now the callback fires continuously as events arrive.

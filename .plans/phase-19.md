# Phase 19: Media Widget

**Status:** Blocked

**Goal:** Add a bar widget that displays the currently playing media
(track title, artist) from system media sources (Spotify, browsers,
etc.) using the Windows media transport controls API.

## Why Blocked

The GSMTC (`GlobalSystemMediaTransportControlsSessionManager`) API
requires a full GUI application process to deliver async callbacks.
`RequestAsync()` returns an `IAsyncOperation` whose completion callback
never fires in console or background daemon processes, regardless of
COM apartment configuration.

### Approaches Tested

| Approach | Result |
|----------|--------|
| Direct `.get()` on a plain thread | Hangs indefinitely |
| `.get()` on an MTA COM thread | Hangs indefinitely |
| `.get()` on an STA COM thread | Hangs indefinitely |
| Manual `SetCompleted` + `MsgWaitForMultipleObjectsEx` (STA, no window) | Times out |
| Manual `SetCompleted` + `MsgWaitForMultipleObjectsEx` (STA, hidden window) | Times out |
| `CoWaitForMultipleHandles` on STA thread | `RPC_E_CALLREJECTED` |
| `.get()` inside a WndProc callback (active message pump) | Hangs (blocks pump) |
| PowerShell 5 subprocess with `AsTask().Wait()` | Hangs indefinitely |
| PowerShell 5 with `System.Windows.Forms.Application.Run()` | Times out |
| PowerShell 7 | No WinRT type support |

### Root Cause

GSMTC async operations require a Windows Runtime dispatcher that is
only present in GUI application processes. The Mosaico daemon uses a
hidden Win32 window with a message pump for system events (window
hooks, hotkeys), but this does not satisfy the WinRT dispatcher
requirement. No combination of COM apartment, hidden window, or manual
message pumping resolves the issue.

### Path Forward

To unblock this phase, one of the following approaches is needed:

1. **Separate GUI helper process** — a small executable (e.g., built
   with `eframe`/`egui`) that queries GSMTC and sends the result to
   the daemon via IPC (named pipe or shared memory).
2. **WinRT dispatcher integration** — use `DispatcherQueue` or
   `CoreDispatcher` from the Windows Runtime, which requires hosting
   a WinRT application context inside the daemon.
3. **Alternative API** — find a non-WinRT way to read media session
   info (e.g., scraping Spotify window titles), though this is
   fragile and app-specific.

## Current State

The media widget infrastructure is fully wired:

- `WidgetConfig::Media` variant exists in the config system
- `BarState::media_text` field is populated on each tick
- `media.rs` has a stub `query_media()` that returns an empty string
- The widget renders correctly when text is present (tested with
  hardcoded values)
- The widget hides itself when `media_text` is empty

Only `query_media()` needs a working GSMTC call to complete the
feature.

## Overview

The media widget uses the Windows `GlobalSystemMediaTransportControls`
API (GSMTC) to read the currently playing media session. It displays
the track title and optionally the artist name in a pill on the status
bar, following the same rendering pattern as existing text-based widgets.

## Reference Design

```
[1] 2 3 4 5 6 7 8  [icon]  BSP       [icon Artist - Track Title]  RAM: 24%  02:15:35
                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^
                                       media widget (right-aligned)
```

When nothing is playing, the widget is invisible (0 width), same
pattern as the Update widget.

## Configuration

```toml
[[right]]
type = "media"
icon = "\uF001"  # music note
# format = "{artist} - {title}"  # optional custom format
# max_length = 40                # truncate long titles
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `true` | Show/hide the widget |
| `icon` | `"\uF001"` | Nerd Font icon |
| `format` | `"{artist} - {title}"` | Display format template |
| `max_length` | `40` | Truncate text beyond this length with "..." |

## Tasks

- [x] Add `Media_Control` and `Foundation` WinRT features to
      `mosaico-windows/Cargo.toml`
- [x] Add `WidgetConfig::Media` variant to `bar.rs`
- [x] Add `media_text: String` to `BarState`
- [x] Create `media.rs` stub in `bar/widgets/`
- [x] Register in `widgets/mod.rs` dispatch
- [x] Add `should_skip` logic for empty `media_text`
- [x] Populate `media_text` in daemon tick handler
- [x] Build, clippy, fmt, test all pass
- [ ] Implement working `query_media()` via GSMTC (BLOCKED)
- [ ] Manual test with Spotify, browser, VLC

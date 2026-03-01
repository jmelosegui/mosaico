# Phase 23: Mouse Follows Focus & Focus Follows Mouse

**Status:** Pending

**Goal:** Add bidirectional mouse/focus integration so the cursor tracks
keyboard-driven focus changes, and optionally, hovering the mouse over a
window focuses it.

## Problem

When the user navigates windows with keyboard shortcuts (Alt+H/J/K/L,
Alt+1-8), only `SetForegroundWindow` is called — the mouse cursor stays
where it was. Many applications use the cursor position to determine
which monitor they open on (e.g., launcher apps, dialog boxes, installer
windows). This means keyboard-only users frequently get new windows
spawning on the wrong monitor.

Moving the cursor to the focused window after every keyboard navigation
solves this and makes keyboard focus equivalent to a mouse click from
the perspective of other applications.

## Features

### Mouse Follows Focus

After every keyboard-driven focus change, move the mouse cursor to the
center of the newly focused window. Disabled when the focus change was
caused by a mouse click (the cursor is already there).

### Focus Follows Mouse

When the mouse enters a managed window, that window receives keyboard
focus automatically — no click required. Two modes:

- **Off** (default): no focus-follows-mouse.
- **On**: hovering any managed window focuses it.

This is the less common feature and should default to off, since it can
be disorienting for users who aren't used to it.

## Configuration

### `config.toml`

```toml
[mouse]
# Move the cursor to the center of the focused window on keyboard
# navigation. Prevents apps from opening on the wrong monitor.
follows_focus = true
# Automatically focus the window under the cursor without clicking.
focus_follows_mouse = false
```

New `[mouse]` section. Both options are independent — users can enable
either, both, or neither.

## Architecture

### Mouse Follows Focus

#### Implementation: single call site

All keyboard-driven focus changes funnel through one method:

```
focus_and_update_border(hwnd) — tiling/mod.rs:648
```

This is called from:
- `focus_direction()` — keyboard navigation (H/J/K/L)
- `goto_workspace()` — workspace switching (Alt+1-8)
- `send_to_workspace()` — send window and follow
- `focus_adjacent_monitor_idx()` — cross-monitor focus
- `handle_event::Created` — new window gets focus

The `Focused` event handler (user clicks a window) also calls it, but
the cursor is already on that window in that case.

**Approach**: Add a `move_cursor: bool` parameter to
`focus_and_update_border()`, or track whether the focus change is
keyboard-driven vs mouse-driven. The simpler approach: add a flag
`skip_cursor_move` that is set `true` only in the `Focused` event
handler (mouse click), and checked in `focus_and_update_border()`.

Alternatively, and even simpler: always move the cursor. When the user
clicks a window, the cursor is already at the center (approximately) —
moving it to exact center is barely noticeable and avoids needing a flag.
However, some users may find the cursor "jumping" annoying if it moves
from where they clicked to the exact center.

**Recommended approach**: Use a boolean field `focus_from_mouse: bool`
set to `true` in the `Focused` event handler before calling
`focus_and_update_border()`, and reset to `false` afterwards. In
`focus_and_update_border()`, skip cursor movement when the flag is set.

#### Win32 API

```rust
use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

/// Moves the cursor to the center of the given window.
fn move_cursor_to_window(hwnd: usize) {
    if let Ok(rect) = Window::from_raw(hwnd).rect() {
        let cx = rect.x + rect.width / 2;
        let cy = rect.y + rect.height / 2;
        unsafe { let _ = SetCursorPos(cx, cy); }
    }
}
```

`SetCursorPos` takes screen coordinates. `Window::rect()` already
returns screen coordinates via `frame::visible_rect()`.

### Focus Follows Mouse

This is more complex. The daemon needs to periodically check the cursor
position and focus the window under it.

#### Option A — Polling with `GetCursorPos` + `WindowFromPoint`

A background thread or the existing 1-second tick queries the cursor
position, finds the window under it, and if it's a managed window on
the active workspace, focuses it.

**Pros**: Simple, no new Win32 hook infrastructure.
**Cons**: Polling interval determines responsiveness. 1 second is too
slow; 100ms is usable but burns CPU. Also can fight with user keyboard
navigation.

#### Option B — Low-level mouse hook (`WH_MOUSE_LL`)

A system-wide mouse hook receives every mouse move. Filter for
`WM_MOUSEMOVE`, call `WindowFromPoint`, focus if managed.

**Pros**: Instant response, no polling.
**Cons**: System-wide hooks are heavyweight. Every mouse movement
triggers a callback. Must be very fast to avoid input lag.

#### Option C — `EVENT_OBJECT_LOCATIONCHANGE` on mouse move (current events)

The event loop already hooks accessibility events. Mouse movement over
a window doesn't generate these events, so this doesn't help.

#### Option D — Timer-based polling with `SetTimer` (Recommended)

Use a Win32 timer on the event loop thread (`SetTimer` with ~100ms
interval). On each `WM_TIMER` tick:

1. `GetCursorPos()` → screen coordinates
2. `WindowFromPoint()` → hwnd under cursor
3. Walk up `GetAncestor(GA_ROOT)` to get the top-level window
4. Check if it's a managed window on the active workspace
5. If different from current focused window → focus it

This runs on the existing message loop thread, no extra thread needed.
The timer only runs when `focus_follows_mouse = true`.

**Debouncing**: Don't focus immediately on every cursor move. Track the
last hwnd under cursor and only focus after it's been the same for 2-3
consecutive ticks (~200-300ms). This prevents rapid focus switching when
moving the mouse across window boundaries.

### New Config Types

In `mosaico-core/src/config/mod.rs`:

```rust
/// Mouse integration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    /// Move the cursor to the center of the focused window on
    /// keyboard navigation.
    pub follows_focus: bool,
    /// Automatically focus the window under the mouse cursor.
    pub focus_follows_mouse: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            follows_focus: true,
            focus_follows_mouse: false,
        }
    }
}
```

Add to `Config`:

```rust
pub struct Config {
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub borders: BorderConfig,
    pub mouse: MouseConfig,
    pub logging: LogConfig,
}
```

### Changes to `TilingManager`

```rust
pub struct TilingManager {
    // ... existing fields ...
    mouse_follows_focus: bool,
    focus_from_mouse: bool,  // suppresses cursor move on click-driven focus
}
```

Updated `focus_and_update_border()`:

```rust
fn focus_and_update_border(&mut self, hwnd: usize) {
    self.focused_window = Some(hwnd);
    self.focused_maximized = Window::from_raw(hwnd).is_maximized();
    Window::from_raw(hwnd).set_foreground();

    if self.mouse_follows_focus && !self.focus_from_mouse {
        move_cursor_to_window(hwnd);
    }
    self.focus_from_mouse = false;

    self.update_border();
}
```

In the `Focused` event handler (mouse click path):

```rust
WindowEvent::Focused { hwnd } => {
    // ... existing logic ...
    self.focus_from_mouse = true;
    self.focus_and_update_border(*hwnd);
}
```

### Changes to config reload

```rust
pub fn reload_config(&mut self, config: &Config) {
    // ... existing reload ...
    self.mouse_follows_focus = config.mouse.follows_focus;
    // focus_follows_mouse is handled by the event loop (timer start/stop)
}
```

### Changes to event loop (focus follows mouse)

The event loop needs to start/stop a `SetTimer` based on the config:

```rust
// In event loop setup, if focus_follows_mouse enabled:
SetTimer(None, FOCUS_FOLLOWS_MOUSE_TIMER_ID, 100, None);

// In WM_TIMER handler:
if w_param == FOCUS_FOLLOWS_MOUSE_TIMER_ID {
    check_cursor_focus();
}
```

The `check_cursor_focus` function queries cursor position and sends a
new internal message (e.g., `DaemonMsg::MouseFocus(hwnd)`) to the
daemon for processing.

### Template update

`generate_config()` adds the new section:

```toml
[mouse]
# Move the cursor to the center of the focused window on keyboard navigation.
follows_focus = true
# Automatically focus the window under the cursor without clicking.
focus_follows_mouse = false
```

## Edge Cases

1. **Multi-monitor DPI**: `SetCursorPos` uses physical screen
   coordinates. `Window::rect()` returns DPI-adjusted coordinates via
   DWM. These should match since both use screen coordinates, but test
   on mixed-DPI setups.

2. **Monocle mode**: Mouse follows focus still applies — the cursor
   moves to the monocle window's center. This is correct because the
   monocle window fills the monitor.

3. **Empty workspace**: No window to focus → no cursor movement.
   `focus_and_update_border` is not called when workspace is empty.

4. **Focus follows mouse vs keyboard navigation**: When the user is
   navigating with keyboard, cursor movement from mouse-follows-focus
   shouldn't trigger focus-follows-mouse on a different window. Guard
   by ignoring `SetCursorPos`-generated movements (track whether we
   just moved the cursor programmatically).

5. **Focus follows mouse + floating dialogs**: Unmanaged windows should
   not steal focus when hovered. Only focus managed windows on the
   active workspace.

6. **User drags a window**: During a drag operation
   (`EVENT_SYSTEM_MOVESIZESTART/END`), focus-follows-mouse should be
   suppressed to avoid fighting the drag.

## Tasks

### Mouse Follows Focus
- [ ] Add `MouseConfig` struct to `mosaico-core/src/config/mod.rs`
- [ ] Add `mouse: MouseConfig` to `Config` struct
- [ ] Add `[mouse]` section to `generate_config()` template
- [ ] Add `mouse_follows_focus: bool` field to `TilingManager`
- [ ] Add `focus_from_mouse: bool` field to `TilingManager`
- [ ] Add `move_cursor_to_window(hwnd)` helper using `SetCursorPos`
- [ ] Update `focus_and_update_border()` to move cursor when enabled
      and `focus_from_mouse` is false
- [ ] Set `focus_from_mouse = true` in `Focused` event handler
- [ ] Update `reload_config()` to propagate `mouse_follows_focus`
- [ ] Pass `mouse_follows_focus` from config in `TilingManager::new()`

### Focus Follows Mouse
- [ ] Add timer infrastructure to event loop (`SetTimer`/`WM_TIMER`)
- [ ] Add `check_cursor_focus()`: `GetCursorPos` + `WindowFromPoint`
      + `GetAncestor(GA_ROOT)` → find managed window under cursor
- [ ] Add debounce logic (2-3 consecutive ticks on same window)
- [ ] Send focus message to daemon when cursor-tracked window changes
- [ ] Guard against programmatic cursor moves retriggering focus
- [ ] Guard against drag operations
- [ ] Start/stop timer based on config reload
- [ ] Pass `focus_follows_mouse` from config to event loop

### General
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: mouse follows focus — keyboard navigate between
      windows, verify cursor moves to center of focused window
- [ ] Manual test: mouse follows focus — click a window, verify cursor
      does NOT jump to center
- [ ] Manual test: mouse follows focus — switch workspaces, verify
      cursor moves to first window on new workspace
- [ ] Manual test: mouse follows focus — cross-monitor focus, verify
      cursor moves to the other monitor
- [ ] Manual test: launch an app that uses cursor position for monitor
      selection, verify it opens on the correct monitor after keyboard
      navigation
- [ ] Manual test: focus follows mouse — hover over windows, verify
      focus follows without clicking
- [ ] Manual test: focus follows mouse — hover over unmanaged window,
      verify no focus change
- [ ] Manual test: both features enabled simultaneously
- [ ] Update documentation (`docs/configuration.md`)
- [ ] Update `.plans/plan.md`

# Phase 21: Configurable Window Hiding Behaviour

**Status:** Complete

**Goal:** Replace the hard-coded `SW_HIDE` strategy with a configurable
window hiding behaviour, adding Cloak (recommended default) and Minimize
as alternatives.

## Overview

When switching workspaces, Mosaico hides windows on the old workspace and
shows windows on the new one. Currently this uses `ShowWindow(hwnd, SW_HIDE)`
unconditionally. While functional, `SW_HIDE` has a known drawback: it
removes the window from the taskbar, which confuses users who expect to
see all running apps in the taskbar regardless of workspace.

The DWM **Cloak** mechanism is an undocumented but widely-used Win32 API
that makes a window invisible without altering its taskbar presence or
window state. Windows itself uses cloaking internally for its virtual
desktop feature. This phase adds three hiding strategies behind a config
option, defaulting to Cloak.

## Hiding Strategies

| Strategy | Win32 API | Taskbar icon | Fires events | Notes |
|----------|-----------|-------------|-------------|-------|
| `"cloak"` | `DwmSetWindowAttribute(DWMWA_CLOAK)` | Kept | No `EVENT_OBJECT_HIDE` | **Recommended.** Invisible but "alive" to the shell. |
| `"hide"` | `ShowWindow(SW_HIDE)` | Removed | `EVENT_OBJECT_HIDE` | Current behaviour. Simple but loses taskbar icons. |
| `"minimize"` | `ShowWindow(SW_MINIMIZE)` | Kept (minimized) | `EVENT_SYSTEM_MINIMIZESTART` | Taskbar icon stays but shows minimized state. Some apps react to minimize (e.g. pause media). |

### Why Cloak is the recommended default

1. **No taskbar disruption** — icons remain, so the taskbar accurately
   reflects running apps. Users can still click a taskbar icon to recall
   a window (Mosaico can handle the resulting focus event).
2. **No spurious events** — cloaking does not fire `EVENT_OBJECT_HIDE`,
   so the `hidden_by_switch` workaround becomes unnecessary for cloaked
   windows. This simplifies event handling.
3. **No app side-effects** — unlike `SW_MINIMIZE`, cloaking doesn't send
   `WM_SIZE` with `SIZE_MINIMIZED`, so apps don't pause playback or
   release resources.
4. **Widely used** — Windows itself uses cloaking for virtual desktops.
   Other popular tiling window managers on Windows use it successfully.

### Cloak API details

**Note:** `DwmSetWindowAttribute(DWMWA_CLOAK=13)` returns `E_ACCESSDENIED`
from user-mode processes. Only system processes (the virtual desktop
manager) can set it directly.

The working approach uses the undocumented **ImmersiveShell COM interface**,
which is the same mechanism Windows uses internally for virtual desktops:

1. `CoCreateInstance(CLSID_ImmersiveShell)` → `IServiceProvider`
2. `IServiceProvider::QueryService(CLSID_IApplicationViewCollection)`
   → `IApplicationViewCollection`
3. `IApplicationViewCollection::GetViewForHwnd(hwnd)` → `IApplicationView`
4. `IApplicationView::SetCloak(1, 2)` to cloak, `SetCloak(1, 0)` to uncloak

COM interface definitions are based on [Ciantic/AltTabAccessor](https://github.com/Ciantic/AltTabAccessor) (MIT licensed),
implemented as raw `#[repr(C)]` vtable structs in `mosaico-windows/src/com/`.

- Works on all Windows 10/11 versions.
- The window remains in the window list (`EnumWindows`), responds to
  messages, and keeps its taskbar button.
- `IsWindowVisible()` still returns `true` for cloaked windows. To check
  cloaked state, use `DwmGetWindowAttribute(DWMWA_CLOAKED=14)` which returns
  a `DWORD` bitmask (non-zero = cloaked).

## Configuration

### `config.toml`

```toml
[layout]
gap = 8
ratio = 0.5
# How windows are hidden during workspace switches.
# "cloak" (recommended): invisible via DWM, keeps taskbar icon.
# "hide": SW_HIDE, removes taskbar icon.
# "minimize": SW_MINIMIZE, keeps taskbar icon but shows as minimized.
hiding = "cloak"
```

The setting lives in `[layout]` because it governs how the tiling layout
engine manages window visibility — it's not a visual/border setting, and
it directly affects workspace switching behaviour.

## Architecture

### New enum: `HidingBehaviour`

In `mosaico-core/src/config/mod.rs`:

```rust
/// How windows are hidden when switching away from their workspace.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HidingBehaviour {
    /// DWM Cloak: window becomes invisible but keeps its taskbar icon
    /// and does not fire EVENT_OBJECT_HIDE. Recommended default.
    #[default]
    Cloak,
    /// ShowWindow(SW_HIDE): window is fully hidden and loses its
    /// taskbar icon. Fires EVENT_OBJECT_HIDE.
    Hide,
    /// ShowWindow(SW_MINIMIZE): window is minimized. Keeps taskbar
    /// icon but shows minimized state. Fires EVENT_SYSTEM_MINIMIZESTART.
    Minimize,
}
```

### Changes to `LayoutConfig`

```rust
pub struct LayoutConfig {
    pub gap: i32,
    pub ratio: f64,
    pub hiding: HidingBehaviour,
}
```

Default: `HidingBehaviour::Cloak`.

### Changes to `Window` (mosaico-windows/src/window.rs)

Add two new methods:

```rust
/// Cloaks the window via DWM, making it invisible without removing
/// its taskbar icon or firing EVENT_OBJECT_HIDE.
pub fn cloak(&self) {
    use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    let value: windows::Win32::Foundation::BOOL = true.into();
    unsafe {
        let _ = DwmSetWindowAttribute(
            self.hwnd,
            windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(13), // DWMWA_CLOAK
            &value as *const _ as *const _,
            std::mem::size_of_val(&value) as u32,
        );
    }
}

/// Uncloaks a previously cloaked window.
pub fn uncloak(&self) {
    use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
    let value: windows::Win32::Foundation::BOOL = false.into();
    unsafe {
        let _ = DwmSetWindowAttribute(
            self.hwnd,
            windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(13), // DWMWA_CLOAK
            &value as *const _ as *const _,
            std::mem::size_of_val(&value) as u32,
        );
    }
}
```

### Changes to `TilingManager`

Store the hiding behaviour:

```rust
pub struct TilingManager {
    // ... existing fields ...
    hiding: HidingBehaviour,
}
```

Passed in via `new()` from the loaded config.

Add a private helper that dispatches based on the strategy:

```rust
/// Hides a window using the configured strategy.
fn hide_window(&self, hwnd: usize) {
    let win = Window::from_raw(hwnd);
    match self.hiding {
        HidingBehaviour::Cloak => win.cloak(),
        HidingBehaviour::Hide => win.hide(),
        HidingBehaviour::Minimize => win.minimize(),
    }
}

/// Shows a window, reversing the configured hiding strategy.
fn show_window(&self, hwnd: usize) {
    let win = Window::from_raw(hwnd);
    match self.hiding {
        HidingBehaviour::Cloak => win.uncloak(),
        HidingBehaviour::Hide => win.show(),
        HidingBehaviour::Minimize => win.show(), // SW_SHOWNOACTIVATE restores minimized too
    }
}
```

### Changes to workspace switching (`workspace.rs`)

Replace direct `Window::from_raw(hwnd).hide()` / `.show()` calls with
`self.hide_window(hwnd)` / `self.show_window(hwnd)`.

In `goto_workspace()`:
```rust
// Before (line 28):
Window::from_raw(hwnd).hide();
// After:
self.hide_window(hwnd);

// Before (line 37):
Window::from_raw(hwnd).show();
// After:
self.show_window(hwnd);
```

Same pattern in `send_to_workspace()`.

### Changes to `hidden_by_switch` handling

The `hidden_by_switch` set is only needed for strategies that fire
events which would otherwise remove windows from management:

- **`Hide`**: fires `EVENT_OBJECT_HIDE` → translated to `Destroyed`.
  Must guard with `hidden_by_switch`.
- **`Minimize`**: fires `EVENT_SYSTEM_MINIMIZESTART` → translated to
  `Minimized`. Must guard with `hidden_by_switch`.
- **`Cloak`**: fires no relevant events. No guard needed.

```rust
// In goto_workspace, before hiding:
if self.hiding != HidingBehaviour::Cloak {
    self.hidden_by_switch.insert(hwnd);
}
self.hide_window(hwnd);

// In goto_workspace, after showing:
if self.hiding != HidingBehaviour::Cloak {
    self.hidden_by_switch.remove(&hwnd);
}
self.show_window(hwnd);
```

For `Minimize` strategy, `EVENT_SYSTEM_MINIMIZESTART` fires. The existing
`Minimized` event handler removes the window from the workspace. Guard
it with `hidden_by_switch`:

```rust
WindowEvent::Minimized { hwnd } => {
    if self.hidden_by_switch.contains(hwnd) {
        return; // programmatic minimize from workspace switch
    }
    // ... existing minimize handling ...
}
```

### Changes to `restore_all_windows()`

On daemon shutdown, all hidden windows must be restored regardless of
strategy. `force_show()` already handles `SW_HIDE` and `SW_MINIMIZE`.
For Cloak, we also need to uncloak:

```rust
pub fn restore_all_windows(&mut self) {
    // ... existing border/monocle cleanup ...
    for mon in &self.monitors {
        for ws in &mon.workspaces {
            for &hwnd in ws.handles() {
                let win = Window::from_raw(hwnd);
                win.uncloak(); // no-op if not cloaked
                win.force_show();
            }
        }
    }
    self.hidden_by_switch.clear();
}
```

Calling `uncloak()` on a non-cloaked window is a no-op (the DWM
attribute just stays `FALSE`), so this is always safe.

### Changes to config reload

When the user changes `hiding` in `config.toml` and hot-reload fires,
the `TilingManager` should update its `hiding` field. No need to
re-hide/show existing windows — the new strategy takes effect on the
next workspace switch.

## Edge Cases

1. **Cloak + `is_visible()` check**: `IsWindowVisible()` returns `true`
   for cloaked windows. This is fine — the window *is* "visible" in
   Win32 terms, just not rendered. Existing `is_visible()` checks
   during window enumeration won't be affected because cloaked windows
   on inactive workspaces are correctly tracked by workspace membership.

2. **User clicks cloaked window's taskbar icon**: Windows will fire
   `EVENT_SYSTEM_FOREGROUND` for that window. The existing `Focused`
   event handler should detect that the window belongs to a different
   workspace and switch to it (or at minimum, uncloak and focus it).
   This needs a new code path in the focus handler.

3. **Switching strategies mid-session**: If a user changes from `cloak`
   to `hide` while windows are cloaked on inactive workspaces, those
   windows remain cloaked until the next workspace switch uncloaks them.
   The `restore_all_windows()` shutdown path always uncloaks, so this
   is safe.

4. **Alt-Tab showing cloaked windows**: Cloaked windows appear in
   Alt-Tab. This is the same behaviour as Windows virtual desktops.
   If the user Alt-Tabs to a cloaked window, it triggers a focus event
   which Mosaico can handle.

## Tasks

- [x] Add `HidingBehaviour` enum to `mosaico-core/src/config/mod.rs`
- [x] Add `hiding: HidingBehaviour` to `LayoutConfig` with `Cloak` default
- [x] Add `hiding = "cloak"` to `generate_config()` template
- [x] Add `cloak()` and `uncloak()` methods to `Window` in `mosaico-windows`
- [x] Add `hiding` field to `TilingManager` struct, passed from config
- [x] Add `hide_window()` and `show_window()` helper methods to `TilingManager`
- [x] Update `goto_workspace()` to use `hide_window()` / `show_window()`
- [x] Update `send_to_workspace()` to use `hide_window()` / `show_window()`
- [x] Update `hidden_by_switch` logic: skip insert/remove for `Cloak`
- [x] Guard `Minimized` event handler with `hidden_by_switch` check
- [x] Update `restore_all_windows()` to call `uncloak()` on all windows
- [x] Handle focus event for cloaked windows on inactive workspaces
      (switch to the window's workspace or uncloak in place)
- [x] Update config reload to propagate `hiding` changes to `TilingManager`
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [x] Run `cargo test` and fix failures
- [x] Manual test: verify `"cloak"` — windows disappear, taskbar icons stay,
      workspace switch is smooth, Alt-Tab works, shutdown restores all
- [ ] Manual test: verify `"hide"` — existing behaviour unchanged
- [ ] Manual test: verify `"minimize"` — windows minimize, taskbar shows
      minimized state, restore works
- [x] Update documentation (`docs/workspaces.md`, `docs/configuration.md`)
- [x] Update `.plans/plan.md`

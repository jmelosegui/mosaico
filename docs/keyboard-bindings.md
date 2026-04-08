# Keyboard Bindings

Mosaico registers global system hotkeys via the Win32 `RegisterHotKey` API.
Hotkey presses are delivered as `WM_HOTKEY` messages in the event loop's
message pump and dispatched as actions to the tiling manager.

## Architecture

Keyboard binding handling spans two crates:

- **mosaico-core** defines platform-agnostic data structures (`Keybinding`,
  `Modifier`) and default bindings
- **mosaico-windows** handles Win32 hotkey registration, virtual key code
  mapping, and `WM_HOTKEY` dispatch

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/keybinding.rs` | `Keybinding`, `Modifier`, `defaults()` |
| `crates/mosaico-windows/src/hotkey.rs` | `HotkeyManager` -- registers/unregisters/dispatches hotkeys |
| `crates/mosaico-windows/src/keys.rs` | `vk_from_name()` -- maps key names to Win32 virtual key codes |

### Key Types

- `Keybinding` -- fields: `action: Action`, `key: String`, `modifiers: Vec<Modifier>`
- `Modifier` -- enum: `Alt`, `Shift`, `Ctrl`, `Win`
- `HotkeyManager` -- fields: `hotkeys: Vec<Hotkey>`, `sender: Sender<Action>`,
  `paused: bool`, `pause_hotkey_id: Option<i32>`
- `Hotkey` (private) -- fields: `id: i32`, `modifiers: HOT_KEY_MODIFIERS`,
  `vk: u32`, `action: Action`

## Registration Flow

1. **Config loading** -- `daemon_loop()` calls `config::merge_missing_keybindings()`,
   which reads `~/.config/mosaico/keybindings.toml`, appends any missing default
   actions (new bindings added by future versions) to the file, then returns the
   merged set. Falls back to `keybinding::defaults()` if the file is absent.

2. **Pass to event loop** -- keybindings are passed to `event_loop::start()`

3. **Create HotkeyManager** -- on the event loop thread, a `HotkeyManager`
   is created with an `mpsc::Sender<Action>`

4. **Per-binding registration** -- `register_from_config()` iterates each
   `Keybinding`:
   - Resolves the key name to a Win32 virtual key code via `vk_from_name()`
   - Converts modifiers: `Alt` -> `MOD_ALT`, `Shift` -> `MOD_SHIFT`,
     `Ctrl` -> `MOD_CONTROL`, `Win` -> `MOD_WIN`
   - Always adds `MOD_NOREPEAT` to prevent key-repeat flooding
   - Calls `RegisterHotKey(None, id, modifiers, vk)` -- `None` HWND means
     registration on the current thread's message queue
   - Stores a `Hotkey { id, modifiers, vk, action }` for later dispatch
   - After the loop, scans for a `TogglePause` action and saves its ID as
     `pause_hotkey_id` so it can be preserved during pause

5. **Cleanup** -- `HotkeyManager` implements `Drop`, calling
   `UnregisterHotKey` for every registered hotkey on shutdown

## Dispatch Flow

```
User presses Alt+J
  -> Windows delivers WM_HOTKEY to thread message queue
  -> GetMessageW() receives the message
  -> msg.message == WM_HOTKEY detected
  -> HotkeyManager::dispatch(msg.wParam as i32)
  -> Finds Hotkey with matching id
  -> Sends action.clone() through mpsc::Sender<Action>
  -> Bridge thread wraps as DaemonMsg::Action
  -> TilingManager::handle_action()
```

## Pause / Unpause

Mosaico can temporarily release all its global hotkeys to the OS so another
application can use those same key combinations.

### How it works

- **`toggle-pause` action** -- when triggered, sends a `WM_APP+2` message
  (`WM_HOTKEY_PAUSE`) to the event loop thread via `PostThreadMessageW`.
- **On pause** -- `HotkeyManager::pause()` calls `UnregisterHotKey` for every
  hotkey *except* the `toggle-pause` binding (so you can always unpause).
  Sets `paused = true`.
- **On unpause** -- `HotkeyManager::unpause()` calls `RegisterHotKey` for every
  previously unregistered hotkey. Sets `paused = false`.
- **State mirroring** -- the daemon main thread tracks a `hotkeys_paused: bool`
  local variable and passes it to the bar so the `paused` widget shows while
  paused.
- **CLI** -- `mosaico pause` / `mosaico unpause` send `PauseHotkeys` /
  `UnpauseHotkeys` IPC commands. The daemon forwards them to the event loop
  via the same `WM_HOTKEY_PAUSE` mechanism.

### Lockout prevention

The `toggle-pause` hotkey stays registered while paused. If no `toggle-pause`
binding is configured, pausing via keyboard isn't possible, but `mosaico unpause`
always works.

### Configuration

Add to `~/.config/mosaico/keybindings.toml`:

```toml
[[keybinding]]
action = "toggle-pause"
key = "P"
modifiers = ["alt", "shift"]
```

Then restart the daemon. The status bar shows a red **PAUSED** indicator while
hotkeys are suspended.

## Key Name Resolution

`vk_from_name()` in `keys.rs` maps string key names to Win32 virtual key
codes. Matching is case-insensitive.

| Key Type | Examples | Resolution |
|----------|----------|------------|
| Single letters | `"A"` - `"Z"` | ASCII value of uppercase letter |
| Digits | `"0"` - `"9"` | ASCII value |
| Function keys | `"F1"` - `"F12"` | `0x70 + n - 1` |
| Named keys | `"Enter"`, `"Tab"`, `"Escape"`, `"Space"` | Match table |
| Arrow keys | `"Left"`, `"Right"`, `"Up"`, `"Down"` | Match table |
| Punctuation | `"Minus"`, `"Plus"`, `"Comma"`, `"Period"` | OEM virtual keys |

Unknown key names return `None` and are logged and skipped (not treated as
errors).

## Default Keybindings

Defined in `keybinding::defaults()` (28 bindings total), using vim-style
spatial motions plus workspace switching:

| Shortcut | Action |
|----------|--------|
| Alt + H | Focus(Left) |
| Alt + J | Focus(Down) |
| Alt + K | Focus(Up) |
| Alt + L | Focus(Right) |
| Alt + Shift + H | Move(Left) |
| Alt + Shift + J | Move(Down) |
| Alt + Shift + K | Move(Up) |
| Alt + Shift + L | Move(Right) |
| Alt + Shift + R | Retile |
| Alt + T | ToggleMonocle |
| Alt + N | CycleLayout |
| Alt + Q | CloseFocused |
| Alt + 1-8 | GoToWorkspace(1-8) |
| Alt + Shift + 1-8 | SendToWorkspace(1-8) |

The H/J/K/L keys follow vim conventions: H=left, J=down, K=up, L=right. The
`toggle-pause` action is not bound by default; add it manually.
Focus navigates spatially; Move swaps or transfers windows in the same
direction. Alt+<number> switches to workspace N on the focused monitor;
Alt+Shift+<number> sends the focused window to workspace N. Alt+N cycles
through available tiling layouts on the current workspace.

## Tests

5 unit tests for `vk_from_name()`: letters (case-insensitive), digits,
named keys, function keys, unknown returns `None`.

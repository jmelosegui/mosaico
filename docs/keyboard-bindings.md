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
- `HotkeyManager` -- fields: `hotkeys: Vec<Hotkey>`, `sender: Sender<Action>`
- `Hotkey` (private) -- fields: `id: i32`, `action: Action`

## Registration Flow

1. **Config loading** -- `daemon_loop()` calls `config::load_keybindings()`,
   which reads `~/.config/mosaico/keybindings.toml` or falls back to
   `keybinding::defaults()`

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
   - Stores a `Hotkey { id, action }` for later dispatch

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

Defined in `keybinding::defaults()` (27 bindings total), using vim-style
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
| Alt + Q | CloseFocused |
| Alt + 1-8 | GoToWorkspace(1-8) |
| Alt + Shift + 1-8 | SendToWorkspace(1-8) |

The H/J/K/L keys follow vim conventions: H=left, J=down, K=up, L=right.
Focus navigates spatially; Move swaps or transfers windows in the same
direction. Alt+N switches to workspace N on the focused monitor;
Alt+Shift+N sends the focused window to workspace N.

## Tests

5 unit tests for `vk_from_name()`: letters (case-insensitive), digits,
named keys, function keys, unknown returns `None`.

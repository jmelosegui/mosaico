# Keyboard Bindings

Mosaico registers a global hotkey for each keybinding. On Windows that is
the `RegisterHotKey` API: hotkey presses are delivered as `WM_HOTKEY`
messages in the message pump of a thread dedicated to hotkeys and
dispatched as actions to the tiling manager.

## Architecture

Keyboard binding handling spans two crates:

- **mosaico-core** defines the bindings (`Keybinding`, `Modifier`) and the
  defaults, parses key names into a `Key` and a `Modifiers` set, and decides
  which hotkeys are registered when (`Hotkeys`), including what stays
  registered while paused. Platforms implement two traits: `HotkeyRegistrar`
  claims single combinations, and `HotkeySource` runs the hotkeys for the
  daemon, the way `EventSource` runs the window events.
- **mosaico-windows** implements `HotkeyRegistrar` with `RegisterHotKey`
  (`Win32Hotkeys`), maps each `Key` to its virtual-key code, and implements
  `HotkeySource` with a thread that dispatches `WM_HOTKEY`
  (`WindowsHotkeySource`)

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/keybinding.rs` | `Keybinding`, `Modifier`, `defaults()`, `Keybinding::combo()` |
| `crates/mosaico-core/src/hotkey/key.rs` | `Key` -- the keys a binding can name; `Letter`, `Digit`, `FunctionKey` |
| `crates/mosaico-core/src/hotkey/key_names.rs` | The names a `Key` is parsed from and shown with |
| `crates/mosaico-core/src/hotkey/combo.rs` | `Modifiers`, `KeyCombo` |
| `crates/mosaico-core/src/hotkey/registrar.rs` | `HotkeyRegistrar` -- the contract platforms implement, `HotkeyId` |
| `crates/mosaico-core/src/hotkey/hotkeys.rs` | `Hotkeys` -- registration, pause, unpause and dispatch |
| `crates/mosaico-core/src/hotkey/rejected.rs` | `Rejected` -- a binding that could not be registered or given back, and why |
| `crates/mosaico-core/src/hotkey/source.rs` | `HotkeySource`, `HotkeyWatch`, `HotkeyControl` -- the hotkey service the daemon starts |
| `crates/mosaico-windows/src/hotkey.rs` | `Win32Hotkeys` -- `RegisterHotKey` and `UnregisterHotKey` |
| `crates/mosaico-windows/src/hotkey_thread.rs` | `WindowsHotkeySource` -- the thread that owns the hotkeys, and its message pump |
| `crates/mosaico-windows/src/keys.rs` | `virtual_key()` -- maps a `Key` to its Win32 virtual-key code |

### Key Types

- `Keybinding` -- fields: `action: Action`, `key: String`, `modifiers: Vec<Modifier>`.
  The key name stays text, so an unknown one puts only its own binding out of
  action; `combo()` parses it into a `KeyCombo`
- `Modifier` -- enum: `Alt`, `Shift`, `Ctrl`, `Win`
- `Key` -- `Letter(Letter)`, `Digit(Digit)`, `Function(FunctionKey)` and the
  named keys (`Enter`, `PageUp`, `Minus`, ...). `Letter`, `Digit` and
  `FunctionKey` can only be built in range (`A`-`Z`, `0`-`9`, `F1`-`F24`)
- `Modifiers` -- a set of modifiers: `CTRL`, `ALT`, `SHIFT`, `WIN`
- `KeyCombo` -- fields: `modifiers: Modifiers`, `key: Key`; shown as `Alt+Shift+J`
- `HotkeyRegistrar` -- trait: `register(id, combo)`, `unregister(id)`
- `Hotkeys` -- the registered bindings, whether they are paused, and the
  pause rule
- `Rejected` -- a binding that could not be registered, or not given back
  for a pause: `action`, `combo`, `error`
- `HotkeySource` -- trait: `start(bindings, actions)` returns a
  `HotkeyWatch`, which has `pause()`, `unpause()` and `stop()`; dropping the
  watch stops the source too. A platform hands its side to the watch as a
  `HotkeyControl`

## Registration Flow

1. **Config loading** -- `daemon_loop()` calls `config::merge_missing_keybindings()`,
   which reads `~/.config/mosaico/keybindings.toml`, appends any missing default
   actions (new bindings added by future versions) to the file, then returns the
   merged set. Falls back to `keybinding::defaults()` if the file is absent.

2. **Start the hotkey source** -- the daemon's `Inputs` pass the keybindings
   to `WindowsHotkeySource.start()` through the `HotkeySource` trait, which
   starts a thread of its own and returns a `HotkeyWatch` once the hotkeys
   are registered

3. **Per-binding registration** -- on the hotkey thread,
   `Hotkeys::register(Win32Hotkeys::new(), &keybindings)` goes through each
   `Keybinding` in order:
   - Parses the key name and modifiers with `Keybinding::combo()`
   - Registers the combination with the binding's position plus one as its
     `HotkeyId`
   - `Win32Hotkeys` maps the key with `virtual_key()` and the modifiers:
     `Alt` -> `MOD_ALT`, `Shift` -> `MOD_SHIFT`, `Ctrl` -> `MOD_CONTROL`,
     `Win` -> `MOD_WIN`
   - Always adds `MOD_NOREPEAT`, so holding the keys down is one press
   - Calls `RegisterHotKey(None, id, modifiers, vk)` -- `None` HWND means
     registration on the current thread's message queue
   - A binding with an unknown key name, or whose combination another program
     holds, is left out, and the hotkey thread logs it at warn, as in
     `Hotkey not working: Alt+J (focus-down): could not register a hotkey: ...`
     (the daemon log is only written with `[logging] enabled = true` in
     `config.toml`)

4. **Cleanup** -- stopping the `HotkeyWatch` on shutdown ends the thread,
   where dropping `Hotkeys` calls `UnregisterHotKey` for every hotkey still
   registered

## Dispatch Flow

```
User presses Alt+J
  -> Windows delivers WM_HOTKEY to the hotkey thread's message queue
  -> The pump takes WM_HOTKEY before anything else in the queue
  -> hotkey_id(msg.wParam) gives the HotkeyId (none for the system's own hotkeys)
  -> Hotkeys::triggered(id) gives the binding's action
     (none while paused for the hotkeys the pause gave back)
  -> Sends the action through mpsc::Sender<Action>
  -> Bridge thread wraps as DaemonMsg::Action
  -> TilingManager::handle_action()
```

## Pause / Unpause

Mosaico can temporarily release all its global hotkeys to the OS so another
application can use those same key combinations.

### How it works

- **`toggle-pause` action** -- when triggered, the daemon calls
  `HotkeyWatch::pause()` or `unpause()`, which posts a `WM_APP+2` message
  (`WM_HOTKEY_PAUSE`) to the hotkey thread via `PostThreadMessageW`, with
  `wParam` 1 to pause and 0 to unpause.
- **On pause** -- `Hotkeys::pause()` unregisters every hotkey *except* the
  ones bound to `toggle-pause` (so you can always unpause). A press of a
  paused hotkey that was already queued does nothing. A hotkey the system
  will not give back stays registered, keeps running its action, and is
  logged as `Hotkey could not be released while paused: ...`.
- **On unpause** -- `Hotkeys::unpause()` registers again exactly the hotkeys
  the pause unregistered. One that another program took in the meantime is
  logged, and tried again at the next unpause.
- **Rule in core** -- which hotkeys survive a pause is decided by `Hotkeys` in
  mosaico-core; a platform's `HotkeyRegistrar` only registers and unregisters.
- **State mirroring** -- the daemon main thread tracks a `hotkeys_paused: bool`
  local variable and passes it to the bar so the `paused` widget shows while
  paused.
- **CLI** -- `mosaico pause` / `mosaico unpause` send `PauseHotkeys` /
  `UnpauseHotkeys` IPC commands. The daemon forwards them through the same
  `HotkeyWatch`.

### Lockout prevention

Every `toggle-pause` hotkey stays registered while paused. If no
`toggle-pause` binding is configured, unpausing via keyboard isn't possible,
but `mosaico unpause` always works.

### Configuration

`toggle-pause` is bound to Alt + Shift + P by default. To use another
combination, change its entry in `~/.config/mosaico/keybindings.toml`:

```toml
[[keybinding]]
action = "toggle-pause"
key = "P"
modifiers = ["alt", "shift"]
```

Then restart the daemon. The status bar shows a red **PAUSED** indicator while
hotkeys are suspended.

## Key Name Resolution

Key names are parsed in mosaico-core (`Key::from_str`), so every platform
accepts the same names. Matching ignores ASCII case. `virtual_key()` in
`keys.rs` then gives the Win32 virtual-key code of the `Key`.

| Key Type | Examples | Virtual-key code |
|----------|----------|------------------|
| Single letters | `"A"` - `"Z"` | ASCII value of uppercase letter |
| Digits | `"0"` - `"9"` | ASCII value |
| Function keys | `"F1"` - `"F24"` | `0x70 + n - 1` |
| Named keys | `"Enter"`, `"Tab"`, `"Escape"`, `"Space"` | Match table |
| Arrow keys | `"Left"`, `"Right"`, `"Up"`, `"Down"` | Match table |
| Punctuation | `"Minus"`, `"Plus"`, `"Comma"`, `"Period"` | OEM virtual keys |

Some keys have a second name: `Return`, `Esc`, `Del`, `Ins`, `PgUp`, `PgDn`,
`Equals` (for `Plus`), `Dot` (for `Period`) and `Grave` (for `Backtick`).

An unknown key name is a config error such as `unknown key name "Foo"
(expected A-Z, 0-9, F1-F24 or a name such as Enter, Space or PageUp)`. The
daemon logs it and skips only that binding, and `mosaico doctor` lists the
unknown names, through the same `Keybinding::combo()`.

## Default Keybindings

Defined in `keybinding::defaults()` (30 bindings total), using vim-style
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
| Alt + M | MinimizeFocused |
| Alt + Shift + P | TogglePause |
| Alt + 1-8 | GoToWorkspace(1-8) |
| Alt + Shift + 1-8 | SendToWorkspace(1-8) |

The H/J/K/L keys follow vim conventions: H=left, J=down, K=up, L=right.
Focus navigates spatially; Move swaps or transfers windows in the same
direction. Alt+<number> switches to workspace N on the focused monitor;
Alt+Shift+<number> sends the focused window to workspace N. Alt+N cycles
through available tiling layouts on the current workspace.

## Tests

- `crates/mosaico-core/src/hotkey/tests/` -- every key name, alias and case
  that parses, unknown names, the ranges of `Letter`, `Digit` and
  `FunctionKey`, modifier sets, `Hotkeys` against a fake registrar (bindings
  left out with the reason, presses mapped to actions, pausing keeps the
  toggle binding registered, a hotkey the system keeps at a pause still
  runs, unpausing restores exactly the paused set, dropping releases what
  is still registered), and `HotkeyWatch` pausing, unpausing and stopping
  its source once
- `crates/mosaico-core/src/config/keybinding_tests.rs` -- every default
  binding has a valid combination
- `crates/mosaico-windows/src/keys_tests.rs` -- every key name registers the
  virtual-key code it always had; `hotkey.rs` checks the modifier flags and
  the ids of the system's own hotkeys; `hotkey_thread.rs` checks that
  stopping the watch ends the thread and drops the action sender

None of them registers a real hotkey.

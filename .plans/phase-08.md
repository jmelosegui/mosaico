# Phase 8: Hotkey System

**Status:** Complete

**Goal:** Allow users to trigger tiling actions (focus, swap, retile) via global
hotkeys and CLI commands. Hotkeys are registered on the Windows message pump
thread and dispatched as `Action` values through a channel to the daemon.

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      action.rs               # Action enum (FocusNext, FocusPrev, SwapNext, SwapPrev, Retile)
      ipc.rs                  # Command::Action variant
  mosaico-windows/
    src/
      hotkey.rs               # HotkeyManager — RegisterHotKey / UnregisterHotKey
      event_loop.rs           # Updated to accept action channel and dispatch WM_HOTKEY
      tiling.rs               # Updated with handle_action, focus_adjacent, swap_adjacent
      daemon.rs               # Updated with DaemonMsg::Action, action bridge thread
  mosaico/
    src/
      main.rs                 # Added Action subcommand group
      commands/
        action.rs             # Sends Action commands to daemon via IPC
```

## Default keybindings

| Shortcut             | Action     |
|----------------------|------------|
| Alt + Shift + J      | FocusNext  |
| Alt + Shift + K      | FocusPrev  |
| Alt + Shift + Enter  | SwapNext   |
| Alt + Ctrl + Enter   | SwapPrev   |
| Alt + Shift + R      | Retile     |

## Tasks

- [x] Define `Action` enum in `mosaico-core`
- [x] Add `Command::Action` variant to IPC protocol
- [x] Implement `HotkeyManager` with `RegisterHotKey` / `UnregisterHotKey`
- [x] Update event loop to accept action channel and dispatch `WM_HOTKEY`
- [x] Add `handle_action`, `focus_adjacent`, `swap_adjacent` to `TilingManager`
- [x] Update daemon with `DaemonMsg::Action` and action bridge thread
- [x] Add CLI `action` subcommand group (focus-next, focus-prev, swap-next, swap-prev, retile)
- [x] Build, lint, test — all passing
- [ ] Commit

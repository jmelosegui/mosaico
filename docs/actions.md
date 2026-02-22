# Action System

Actions represent user-initiated operations on the tiling manager. They are the
unifying interface between hotkeys and CLI commands -- both paths produce an
`Action` that is dispatched to `TilingManager::handle_action()`.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/action.rs` | `Action` enum definition |
| `crates/mosaico-windows/src/tiling.rs` | `TilingManager::handle_action()` -- executes actions |
| `crates/mosaico/src/main.rs` | CLI `ActionCommands` enum mapping subcommands to actions |
| `crates/mosaico/src/commands/action.rs` | Sends actions to the daemon over IPC |

### Key Types

- `Action` (enum) -- derives `Debug`, `Clone`, `PartialEq`, `Serialize`,
  `Deserialize` with `#[serde(rename_all = "kebab-case")]`

## Available Actions

| Action | Description | Default Hotkey |
|--------|-------------|----------------|
| `FocusNext` | Move focus to next window in workspace | Alt+J |
| `FocusPrev` | Move focus to previous window | Alt+K |
| `SwapNext` | Swap focused window with next in order | Alt+Shift+J |
| `SwapPrev` | Swap focused window with previous | Alt+Shift+K |
| `Retile` | Re-apply layout on all monitors | Alt+Shift+R |
| `FocusMonitorNext` | Focus right spatial neighbor or next monitor | Alt+L |
| `FocusMonitorPrev` | Focus left spatial neighbor or previous monitor | Alt+H |
| `MoveToMonitorNext` | Move window right or to next monitor | Alt+Shift+L |
| `MoveToMonitorPrev` | Move window left or to previous monitor | Alt+Shift+H |
| `ToggleMonocle` | Toggle monocle mode on focused monitor | Alt+T |
| `CloseFocused` | Close the focused window via `WM_CLOSE` | Alt+Q |

## Trigger Paths

### Hotkey Path

```
Key press -> WM_HOTKEY -> HotkeyManager::dispatch() -> mpsc channel
  -> bridge thread -> DaemonMsg::Action -> TilingManager::handle_action()
```

### CLI Path

```
mosaico action <verb> -> Command::Action { action } -> named pipe
  -> ipc_loop -> DaemonMsg::Command -> TilingManager::handle_action()
```

Both paths converge at `TilingManager::handle_action()`.

## Execution

`TilingManager::handle_action()` matches each variant and calls the
corresponding method:

| Action | Method | File:Line |
|--------|--------|-----------|
| `FocusNext` | `focus_adjacent(1)` | `tiling.rs:190` |
| `FocusPrev` | `focus_adjacent(-1)` | `tiling.rs:190` |
| `SwapNext` | `swap_adjacent(1)` | `tiling.rs:217` |
| `SwapPrev` | `swap_adjacent(-1)` | `tiling.rs:217` |
| `Retile` | `retile_all()` | `tiling.rs:363` |
| `FocusMonitorNext` | `focus_monitor(1)` | `tiling.rs:233` |
| `FocusMonitorPrev` | `focus_monitor(-1)` | `tiling.rs:233` |
| `MoveToMonitorNext` | `move_to_monitor(1)` | `tiling.rs:332` |
| `MoveToMonitorPrev` | `move_to_monitor(-1)` | `tiling.rs:332` |
| `ToggleMonocle` | `toggle_monocle()` | `tiling.rs:353` |
| `CloseFocused` | `close_focused()` | `tiling.rs:174` |

## Serialization

Actions use `#[serde(rename_all = "kebab-case")]` so they serialize as
human-readable strings:

- `FocusNext` -> `"focus-next"`
- `MoveToMonitorPrev` -> `"move-to-monitor-prev"`

This format is used in both TOML configuration files (keybindings) and JSON
IPC messages.

## Design Decisions

- Actions are a flat enum rather than a nested structure, keeping dispatch
  simple and serialization straightforward.
- Direction-based methods (`focus_adjacent`, `swap_adjacent`, `focus_monitor`,
  `move_to_monitor`) take `+1` or `-1` to avoid duplicating logic for
  next/prev variants.
- `CloseFocused` sends `WM_CLOSE` rather than forcefully terminating the
  process, giving the target window a chance to save state or prompt the user.

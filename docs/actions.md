# Action System

Actions represent user-initiated operations on the tiling manager. They are the
unifying interface between hotkeys and CLI commands -- both paths produce an
`Action` that is dispatched to `TilingManager::handle_action()`.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/action.rs` | `Action` enum, `Direction` enum |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-windows/src/tiling/mod.rs` | `TilingManager::handle_action()` -- executes actions |
| `crates/mosaico-windows/src/tiling/navigation.rs` | `focus_direction()`, `move_direction()` |
| `crates/mosaico-windows/src/tiling/workspace.rs` | `goto_workspace()`, `send_to_workspace()` |
| `crates/mosaico/src/main.rs` | CLI `ActionCommands` / `DirectionCommands` mapping |
| `crates/mosaico/src/commands/action.rs` | Sends actions to the daemon over IPC |

### Key Types

- `Direction` (enum) -- `Left`, `Right`, `Up`, `Down`. Implements `FromStr`,
  `Display`, and `Copy`.
- `Action` (enum) -- derives `Debug`, `Clone`, `PartialEq`, `Serialize`,
  `Deserialize`. Uses custom `FromStr`/`Display`/`TryFrom<String>`/`Into<String>`
  for serde instead of the older `rename_all` attribute.

## Direction

The `Direction` enum represents spatial directions for focus and move actions:

```rust
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
```

Directions parse from and serialize to lowercase strings: `"left"`, `"right"`,
`"up"`, `"down"`.

## Available Actions

| Action | Description | Default Hotkey |
|--------|-------------|----------------|
| `Focus(Left)` | Focus window to the left (crosses monitors) | Alt+H |
| `Focus(Right)` | Focus window to the right (crosses monitors) | Alt+L |
| `Focus(Up)` | Focus window above (same monitor only) | Alt+K |
| `Focus(Down)` | Focus window below (same monitor only) | Alt+J |
| `Move(Left)` | Swap/move window left (crosses monitors) | Alt+Shift+H |
| `Move(Right)` | Swap/move window right (crosses monitors) | Alt+Shift+L |
| `Move(Up)` | Swap window up (same monitor only) | Alt+Shift+K |
| `Move(Down)` | Swap window down (same monitor only) | Alt+Shift+J |
| `Retile` | Re-apply layout on all monitors | Alt+Shift+R |
| `ToggleMonocle` | Toggle monocle mode on focused monitor | Alt+T |
| `CloseFocused` | Close the focused window via `WM_CLOSE` | Alt+Q |
| `GoToWorkspace(1-8)` | Switch to workspace N on focused monitor | Alt+1 - Alt+8 |
| `SendToWorkspace(1-8)` | Send focused window to workspace N | Alt+Shift+1 - Alt+Shift+8 |

### Horizontal vs Vertical Behavior

- **Left/Right**: spatial horizontal neighbor lookup. If no neighbor exists on
  the current monitor, overflows to the physically adjacent monitor.
- **Up/Down**: spatial vertical neighbor lookup. Stops at the monitor boundary
  (no cross-monitor overflow).

## Trigger Paths

### Hotkey Path

```
Key press -> WM_HOTKEY -> HotkeyManager::dispatch() -> mpsc channel
  -> bridge thread -> DaemonMsg::Action -> TilingManager::handle_action()
```

### CLI Path

```
mosaico action focus left -> Command::Action { action } -> named pipe
  -> ipc_loop -> DaemonMsg::Command -> TilingManager::handle_action()
```

Both paths converge at `TilingManager::handle_action()`.

## Execution

`TilingManager::handle_action()` matches each variant:

| Action | Method |
|--------|--------|
| `Focus(dir)` | `focus_direction(dir)` (in `navigation.rs`) |
| `Move(dir)` | `move_direction(dir)` (in `navigation.rs`) |
| `Retile` | `retile_all()` (in `layout.rs`) |
| `ToggleMonocle` | `toggle_monocle()` (in `layout.rs`) |
| `CloseFocused` | `close_focused()` |
| `GoToWorkspace(n)` | `goto_workspace(n)` (in `workspace.rs`) |
| `SendToWorkspace(n)` | `send_to_workspace(n)` (in `workspace.rs`) |

`focus_direction` and `move_direction` branch on horizontal vs vertical
internally, using `resolve_horizontal_target()` for Left/Right and
`find_same_monitor_neighbor()` for Up/Down.

## Serialization

Actions use custom `FromStr`/`Display` implementations for serde:

- `Focus(Left)` -> `"focus-left"`
- `Move(Right)` -> `"move-right"`
- `Retile` -> `"retile"`
- `ToggleMonocle` -> `"toggle-monocle"`
- `CloseFocused` -> `"close-focused"`
- `GoToWorkspace(3)` -> `"goto-workspace-3"`
- `SendToWorkspace(1)` -> `"send-to-workspace-1"`

Workspace numbers are validated to the range 1-8 during parsing. This format
is used in TOML configuration files (keybindings) and JSON IPC messages.

## Design Decisions

- `Focus` and `Move` each take a `Direction` parameter rather than having
  separate `FocusNext`/`FocusPrev`/`FocusMonitorNext`/etc. variants. This
  consolidates direction logic in one place and eliminates the distinction
  between within-workspace and cross-monitor actions.
- The spatial navigation algorithm lives in `mosaico-core/src/spatial.rs` as
  pure functions over `(handle, Rect)` slices, making it testable without
  Win32 dependencies.
- `CloseFocused` sends `WM_CLOSE` rather than forcefully terminating the
  process, giving the target window a chance to save state or prompt the user.
- Left/Right overflow to adjacent monitors because horizontal monitor
  arrangement is the common physical layout. Up/Down stop at boundaries
  because vertical monitor stacking is less common and overflowing would be
  confusing.

## Tests

- `roundtrip_all_actions` -- verifies `Display`/`FromStr` roundtrip for all
  action variants (including workspace actions)
- `unknown_action_returns_error` -- invalid action strings produce errors
- `unknown_direction_returns_error` -- invalid direction strings produce errors
- `serde_roundtrip` -- JSON serialization roundtrip

# Action System

Actions represent user-initiated operations on the tiling manager. They are the
unifying interface between hotkeys, CLI commands and the pointer on the bar --
every path produces an `Action` that is dispatched to the tiling manager, with
an `Origin` that says where it came from.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/action.rs` | `Action` enum, `Direction` enum |
| `crates/mosaico-core/src/action_origin.rs` | `Origin`: a hotkey or the CLI, or the bar of a monitor; `Scope` and `Action::scope()`: what an action acts on |
| `crates/mosaico-core/src/action_step.rs` | `workspace_after()`: the workspace a step through them lands on |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-core/src/tiler/manager.rs` | `TilingManager::handle_action()` and `handle_action_from()` -- execute actions |
| `crates/mosaico-core/src/tiler/actions.rs` | The tiler's dispatch of each action, from its origin, and the steps through the workspaces |
| `crates/mosaico-core/src/tiler/navigation.rs` | `focus_direction()`, `move_direction()` |
| `crates/mosaico-core/src/tiler/workspace.rs` | `goto_workspace()`, `send_to_workspace()` |
| `crates/mosaico/src/action_commands.rs` | CLI `ActionCommands` / `DirectionCommands` mapping |
| `crates/mosaico/src/commands/action.rs` | Sends actions to the daemon over IPC |

### Key Types

- `Direction` (enum) -- `Left`, `Right`, `Up`, `Down`. Implements `FromStr`,
  `Display`, and `Copy`.
- `Action` (enum) -- derives `Debug`, `Clone`, `PartialEq`, `Serialize`,
  `Deserialize`. Uses custom `FromStr`/`Display`/`TryFrom<String>`/`Into<String>`
  for serde instead of the older `rename_all` attribute.
- `Origin` (enum) -- `Focused`, for a hotkey or the CLI, which act on the
  focused monitor, and `Bar(monitor)`, for a click or a scroll on the bar of
  the monitor at that index. `monitor()` and `from_pointer()` read it.
- `Scope` (enum) -- what `Action::scope()` says an action acts on:
  `FocusedWindow` (`Focus`, `Move`, `SendToWorkspace`, `CloseFocused`,
  `MinimizeFocused`), `Monitor` (`CycleLayout`, `ToggleMonocle` and the
  workspace switches) or `Daemon` (`Retile`, `TogglePause`). The tiler moves
  focus to a bar's monitor first for a `Monitor` action, and a bar may only
  run `Monitor` and `Daemon` actions.

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
| `CycleLayout` | Cycle to the next layout on the focused monitor | Alt+N |
| `CloseFocused` | Close the focused window via `WM_CLOSE` | Alt+Q |
| `GoToWorkspace(1-8)` | Switch to workspace N on focused monitor | Alt+1 - Alt+8 |
| `GoToWorkspaceNext` | Switch to the workspace after the one on screen | -- |
| `GoToWorkspacePrev` | Switch to the workspace before the one on screen | -- |
| `SendToWorkspace(1-8)` | Send focused window to workspace N | Alt+Shift+1 - Alt+Shift+8 |

`GoToWorkspaceNext` and `GoToWorkspacePrev` step from the workspace on the
focused monitor, every monitor in global mode, the way `workspace_after()`
says: past the last workspace a step goes back to the first, and before the
first to the last, with `[workspaces] wrap = true`; without it, a step at
either end does nothing. They have no default hotkey; bind them in
`keybindings.toml`.

### Horizontal vs Vertical Behavior

- **Left/Right**: spatial horizontal neighbor lookup. If no neighbor exists on
  the current monitor, overflows to the physically adjacent monitor.
- **Up/Down**: spatial vertical neighbor lookup. Stops at the monitor boundary
  (no cross-monitor overflow).

## Trigger Paths

### Hotkey Path

```
Key press -> WM_HOTKEY -> Hotkeys::triggered() -> mpsc channel
  -> bridge thread -> DaemonMsg::Action -> TilingManager::handle_action()
```

### CLI Path

```
mosaico action focus left -> Command::Action { action } -> named pipe
  -> ipc_loop -> DaemonMsg::Command -> TilingManager::handle_action()
```

### Bar Path

```
Click or wheel on a bar -> bar window procedure -> PointerSink
  -> DaemonMsg::BarPointer -> BarManager::pointer() / take_wheel()
  -> (Action, Origin::Bar(monitor)) -> TilingManager::handle_action_from()
```

The bar manager turns a left click on a workspace label, and the wheel once
it goes quiet, into `GoToWorkspace(n)` from the bar's monitor, and a click on
a widget into the action its button is bound to there by `[[mouse.click]]`,
which may only be a `Monitor` or a `Daemon` action; see
[status-bar.md](status-bar.md#pointer-input).

Every path goes through `carry_out()` in `daemon_loop_commands.rs`, so an
action does the same from a hotkey, the CLI and the bar. `toggle-pause` is
handled there, `retile` re-reads the work areas there before retiling, and
every other action converges at `TilingManager::handle_action_from()`, with
`Origin::Focused` from a hotkey or the CLI.

### Origins

An action from a bar acts on the bar's monitor. One that acts on a monitor
(`Scope::Monitor`: `CycleLayout`, `ToggleMonocle` and the workspace switches)
moves focus to that monitor first when it does not have it, to the window its
workspace takes focus back with, or parks it there when the workspace is
empty, then does what its key does: in global mode every monitor switches,
and focus goes to the clicked monitor. A `Retile` from a bar retiles every
monitor and leaves focus where it is. A bar whose monitor the tiler has no
state for does nothing. Nothing the tiler does for an action from the bar
moves the pointer: the step's focus changes carry `move_cursor: false` and it
puts no `CenterCursor`, whatever `[mouse] follows_focus` says.

An action fails only when something it needed reported an error, such as a
`retile` that could not read the monitors (it still retiles with the work
areas it last saw). The daemon logs the failure whoever sent the action, and
the CLI also exits with status 1 and prints why. An action with nothing to act
on, such as focusing left at the left edge or closing with nothing focused,
does nothing and succeeds.

## Execution

`TilingManager::handle_action_from()` hands the action and its origin to the
tiler (`handle_action()` does with `Origin::Focused`), and
`Tiler::handle_action()` (in `crates/mosaico-core/src/tiler/actions.rs`)
first moves focus to the origin's monitor when it should (see
[Origins](#origins)), then matches each variant:

| Action | Method |
|--------|--------|
| `Focus(dir)` | `focus_direction(dir)` (in `navigation.rs`) |
| `Move(dir)` | `move_direction(dir)` (in `navigation.rs`) |
| `Retile` | `retile_all()` (in `layout_actions.rs`) |
| `ToggleMonocle` | `toggle_monocle()` (in `layout_actions.rs`) |
| `CycleLayout` | `cycle_layout()` (in `layout_actions.rs`) |
| `CloseFocused` | asks the platform to close the focused window |
| `GoToWorkspace(n)` | `goto_workspace(n)` (in `workspace.rs`) |
| `GoToWorkspaceNext` / `GoToWorkspacePrev` | `step_workspace(1)` / `step_workspace(-1)` (in `actions.rs`) |
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
- `CycleLayout` -> `"cycle-layout"`
- `CloseFocused` -> `"close-focused"`
- `GoToWorkspace(3)` -> `"goto-workspace-3"`
- `GoToWorkspaceNext` -> `"goto-workspace-next"`
- `GoToWorkspacePrev` -> `"goto-workspace-prev"`
- `SendToWorkspace(1)` -> `"send-to-workspace-1"`

Workspace numbers are validated to the range 1-8 during parsing; `next` and
`prev` are matched before a number is. This format is used in TOML
configuration files (keybindings) and JSON IPC messages.

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
- `stepping_through_workspaces_is_named_next_and_prev` -- the names of
  `GoToWorkspaceNext` and `GoToWorkspacePrev`
- `action_step_tests.rs` -- where `workspace_after()` lands, with and without
  wrap, at either end and for several steps
- `action_scope_tests.rs` -- what each action acts on
- `tiler/tests/bar_click_tests.rs`, `bar_click_focus_tests.rs`,
  `bar_widget_tests.rs`, `workspace_step_tests.rs` and
  `workspace_wrap_tests.rs` -- actions from a bar, a widget's included, and
  stepping through the workspaces, in the tiler

# Workspaces

Each monitor in Mosaico supports up to 8 independent workspaces. Only one
workspace is active per monitor at a time. Switching workspaces hides the
current windows and shows the target workspace's windows.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/action.rs` | `GoToWorkspace(u8)`, `SendToWorkspace(u8)`, `MAX_WORKSPACES` |
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered window handle collection |
| `crates/mosaico-core/src/config/keybinding.rs` | Default workspace keybindings |
| `crates/mosaico-windows/src/tiling/mod.rs` | `MonitorState` with `Vec<Workspace>`, `active_workspace` |
| `crates/mosaico-windows/src/tiling/workspace.rs` | `goto_workspace()`, `send_to_workspace()` |

### Key Types

- `MonitorState` -- per-monitor state holding `Vec<Workspace>` (8 workspaces),
  `active_workspace: usize` (0-indexed), and `monocle: bool`
- `MAX_WORKSPACES: u8 = 8` -- public constant in `mosaico-core`

## Actions

| Action | Description | Default Hotkey |
|--------|-------------|----------------|
| `GoToWorkspace(1)` - `GoToWorkspace(8)` | Switch to workspace N | Alt+1 - Alt+8 |
| `SendToWorkspace(1)` - `SendToWorkspace(8)` | Send focused window to workspace N | Alt+Shift+1 - Alt+Shift+8 |

Actions use 1-based indexing in the user-facing interface (config, CLI) and
0-based indexing internally.

## Switching Workspaces

`goto_workspace(n)` (in `tiling/workspace.rs`):

1. If already on workspace N, does nothing
2. Hides all windows in the current workspace (calls `ShowWindow(SW_HIDE)`)
3. Adds hidden windows to `hidden_by_switch` set to prevent the `Destroyed`
   event handler from removing them
4. Sets `active_workspace` to N
5. Shows all windows in the target workspace (calls `ShowWindow(SW_SHOW)`)
6. Removes shown windows from `hidden_by_switch`
7. Retiles the monitor
8. Focuses the first window in the new workspace

### Hidden Window Tracking

When windows are hidden during a workspace switch, the OS fires
`EVENT_OBJECT_HIDE` which normally maps to a `Destroyed` event. Without
special handling, this would cause the tiling manager to remove those windows
from the workspace permanently.

The `hidden_by_switch: HashSet<usize>` field tracks windows that were
programmatically hidden. When a `Destroyed` event arrives:

- If the handle is in `hidden_by_switch`, the event is ignored
- If the handle is not in the set, normal destroy handling proceeds

## Sending Windows

`send_to_workspace(n)` (in `tiling/workspace.rs`):

1. Removes the focused window from the current workspace
2. Adds it to workspace N
3. Hides the window and adds it to `hidden_by_switch`
4. Retiles the current workspace
5. Focuses the next available window

The sent window will appear when the user switches to workspace N.

## Workspace Initialization

At daemon startup, all 8 workspaces are created per monitor (as empty
`Workspace` instances). Existing visible windows are added to workspace 1
(index 0) of their respective monitors. Only workspace 1 is active initially.

## Bar Integration

The status bar displays workspace indicators showing which workspaces
contain windows and which is active. The `BarState` struct provides:

- `active_workspace` -- 0-indexed active workspace
- `workspace_count` -- number of workspaces that contain at least one window

The workspaces widget renders one pill per populated workspace, with the
active workspace highlighted in the configured `active_workspace` color.

## Configuration

Default keybindings are generated in a loop in `keybinding::defaults()`:

```toml
# Alt+1..8 switches to workspace 1..8
[[keybinding]]
action = "goto-workspace-1"
key = "1"
modifiers = ["alt"]

# Alt+Shift+1..8 sends focused window to workspace 1..8
[[keybinding]]
action = "send-to-workspace-1"
key = "1"
modifiers = ["alt", "shift"]
```

## Serialization

Actions serialize as `"goto-workspace-N"` and `"send-to-workspace-N"` where
N is 1-8. Parsing validates the range and rejects invalid workspace numbers.

## Design Decisions

- **8 workspaces per monitor** is a fixed maximum (`MAX_WORKSPACES`). This
  simplifies the implementation and matches common usage patterns. All 8
  are pre-allocated at startup.
- **1-indexed user interface** (Alt+1 = workspace 1) is conventional and
  intuitive. Internal 0-based indexing is an implementation detail.
- **Hide/show via `ShowWindow`** is the simplest approach for workspace
  switching. Windows retain their positions and are simply made invisible.
- **`hidden_by_switch` tracking** is essential to prevent the tiling manager
  from interpreting programmatic hides as window closures.
- **Per-monitor workspaces** rather than global workspaces match the
  multi-monitor mental model where each monitor is independent.
- **Workspace 1 is always the initial active workspace**. New windows
  created after startup go to the active workspace of the focused monitor.

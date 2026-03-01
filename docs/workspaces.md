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
2. Hides all windows in the current workspace using the configured hiding
   strategy (see below)
3. For `Hide` and `Minimize` strategies: adds hidden windows to
   `hidden_by_switch` set to prevent spurious event handlers from removing
   them. Skipped for `Cloak` because cloaking does not fire events.
4. Sets `active_workspace` to N
5. Shows all windows in the target workspace (reverses the hiding strategy)
6. For `Hide` and `Minimize` strategies: removes shown windows from
   `hidden_by_switch`
7. Retiles the monitor
8. Focuses the first window in the new workspace

### Hiding Behaviour

The `hiding` setting in `[layout]` controls how windows are hidden during
workspace switches. Three strategies are available:

| Strategy | Method | Taskbar Icon | Fires Events |
|----------|--------|-------------|-------------|
| `"cloak"` (default) | COM ImmersiveShell cloaking | Kept | No |
| `"hide"` | `ShowWindow(SW_HIDE)` | Removed | `EVENT_OBJECT_HIDE` |
| `"minimize"` | `ShowWindow(SW_MINIMIZE)` | Kept (minimized) | `EVENT_SYSTEM_MINIMIZESTART` |

**Cloak** is the recommended default. It uses the same undocumented COM
interface (`IApplicationView::SetCloak`) that Windows uses internally for
virtual desktops. Windows become invisible but keep their taskbar icons and
do not fire any events that would confuse the tiling manager.

The implementation lives in `crates/mosaico-windows/src/com/` and uses raw
`#[repr(C)]` vtable structs for the ImmersiveShell COM interfaces.

Note: `DwmSetWindowAttribute(DWMWA_CLOAK=13)` returns `E_ACCESSDENIED` from
user-mode processes. Only the COM approach works.

### Hidden Window Tracking

When using `Hide` or `Minimize` strategies, the OS fires events
(`EVENT_OBJECT_HIDE` or `EVENT_SYSTEM_MINIMIZESTART`) that would normally
cause the tiling manager to remove windows from the workspace.

The `hidden_by_switch: HashSet<usize>` field tracks windows that were
programmatically hidden. When a `Destroyed` or `Minimized` event arrives:

- If the handle is in `hidden_by_switch`, the event is ignored
- If the handle is not in the set, normal handling proceeds

This tracking is skipped for `Cloak` mode because cloaking does not fire
any relevant events.

### Cloaked Window Focus Handling

When using Cloak mode, cloaked windows retain their taskbar icons. If a user
clicks a cloaked window's taskbar icon, Windows fires `EVENT_SYSTEM_FOREGROUND`.
The `Focused` event handler detects that the window belongs to an inactive
workspace and automatically switches to that workspace.

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
- **Configurable hiding strategy** allows choosing between Cloak (default),
  Hide, and Minimize. Cloak preserves taskbar icons and avoids spurious
  events. Hide is the legacy approach. Minimize keeps taskbar icons but
  may trigger app-specific behaviour (e.g. pausing media).
- **`hidden_by_switch` tracking** is essential for Hide and Minimize modes
  to prevent the tiling manager from interpreting programmatic hides as
  window closures. Skipped for Cloak mode which fires no events.
- **Per-monitor workspaces** rather than global workspaces match the
  multi-monitor mental model where each monitor is independent.
- **Workspace 1 is always the initial active workspace**. New windows
  created after startup go to the active workspace of the focused monitor.

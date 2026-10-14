# Workspaces

Mosaico supports up to 8 workspaces. Each monitor has its own per-workspace
window list; windows never migrate between monitors as part of a switch.
Only one workspace is active per monitor at a time. Switching workspaces
hides the current windows and shows the target workspace's windows.

The `workspaces.mode` config field controls how a switch propagates across
monitors:

- `"per-monitor"` (default) -- only the focused monitor's
  `active_workspace` advances.
- `"global"` -- every monitor's `active_workspace` advances to the same
  index in lockstep, mirroring Windows virtual desktops.

A config reload that turns `"global"` on switches every monitor to the
focused monitor's workspace, the way a switch does, so they start out in
step.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/action.rs` | `GoToWorkspace(u8)`, `SendToWorkspace(u8)`, `MAX_WORKSPACES` |
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered window handle collection |
| `crates/mosaico-core/src/config/keybinding.rs` | Default workspace keybindings |
| `crates/mosaico-core/src/tiler/state.rs` | `MonitorState` with `Vec<Workspace>` and `active_workspace` |
| `crates/mosaico-core/src/tiler/mod.rs` | `Tiler.workspace_mode` |
| `crates/mosaico-core/src/tiler/workspace.rs` | `goto_workspace()`, `goto_workspace_on()`, `send_to_workspace()` |
| `crates/mosaico-core/src/tiler/hidden_how.rs` | `HiddenHow`: how each window the tiler hid was hidden, to show it the same way |
| `crates/mosaico-core/src/config/types.rs` | `WorkspacesConfig`, `WorkspaceMode` |

### Key Types

- `MonitorState` -- per-monitor state holding `Vec<Workspace>` (8 workspaces),
  `active_workspace: usize` (0-indexed), and `monocle: bool`
- `MAX_WORKSPACES: u8 = 8` -- public constant in `mosaico-core`
- `WorkspacesConfig` -- top-level `[workspaces]` section: `mode: WorkspaceMode`
  and `layouts: HashMap<u8, LayoutKind>` (per-workspace layout overrides)
- `WorkspaceMode` -- `PerMonitor` (default) or `Global`

## Actions

| Action | Description | Default Hotkey |
|--------|-------------|----------------|
| `GoToWorkspace(1)` - `GoToWorkspace(8)` | Switch to workspace N | Alt+1 - Alt+8 |
| `SendToWorkspace(1)` - `SendToWorkspace(8)` | Send focused window to workspace N | Alt+Shift+1 - Alt+Shift+8 |

Actions use 1-based indexing in the user-facing interface (config, CLI) and
0-based indexing internally.

## Switching Workspaces

`goto_workspace(n)` (in `crates/mosaico-core/src/tiler/workspace.rs`) dispatches on
`workspace_mode`:

- `PerMonitor` -- calls `goto_workspace_on(focused_monitor, n, refocus=true)`.
- `Global` -- iterates every monitor and calls `goto_workspace_on(i, n,
  refocus = (i == focused_monitor))`. Only the previously-focused monitor
  takes keyboard focus; the other monitors retile silently.

`goto_workspace_on(mon_idx, idx, refocus)` performs the per-monitor work:

1. If the monitor is already on workspace `idx`, does nothing
2. Remembers the focused window as the left workspace's `last_focused`
   (nothing when `refocus` is false)
3. Sets `active_workspace` to `idx`
4. Shows all windows in the target workspace, each the way it was hidden
   (see below)
5. Retiles the monitor
6. When `refocus` is true, focuses the monocle window, the remembered
   `last_focused` or the first window of the new workspace; otherwise leaves
   keyboard focus alone
7. Hides all windows of the workspace left. Showing the target first means
   the window that held focus is not hidden while nothing else can take it,
   which would let the OS show it again during rapid switching.
8. When `refocus` is true and the new workspace is empty, pins focus to the
   monitor until something takes it

A display change can change a workspace while it is out of view. Windows
from a monitor that went away can wait there, behind its monocle window if it
is in monocle, and are put in place when the workspace is switched to. If its
monocle window went back to its own monitor, the workspace leaves monocle
when other windows remain, unless windows wait behind it there: the one in
front of them becomes the monocle window (see
[monitor-management.md](monitor-management.md#disconnect-and-reconnect)).

### Hiding Behaviour

The `hiding` setting in `[layout]` controls how windows are hidden during
workspace switches. `HidingBehaviour` describes each value by what the user
sees:

| Strategy | What the user sees | The desktop reports the window |
|----------|--------------------|--------------------------------|
| `"cloak"` (default) | Invisible, keeps its taskbar icon | Shown, not minimized |
| `"hide"` | Fully hidden, taskbar icon removed | Not shown |
| `"minimize"` | Minimized, taskbar icon kept | Minimized |

Which strategies a platform has, and which window events each causes, is the
platform's to say: `WindowControl::hiding_events` answers, for each one,
whether hiding a window that way reports it hidden or minimized, and whether
showing it again reports it restored (`HidingEvents`), or `None` when the
platform does not have it. The daemon asks once, at startup, and keeps the
answers in a `HidingSupport`.

When the platform does not have the configured strategy, the tiler uses the
first it has of cloak, hide and minimize instead, and logs a warning
(`HidingSupport::choose`). It chooses at startup, and again only when a config
reload changes `hiding`, so the warning is not repeated on every edit. A
platform with no strategy at all keeps the configured one, also with a
warning. The hide and show effects carry the strategy really used, so windows
are shown again, their events recognized and the record of hidden windows
written the way they were hidden.

The tiler remembers how it hid each window
(`crates/mosaico-core/src/tiler/hidden_how.rs`) and shows it again that way,
so changing `hiding` while windows sit on other workspaces only affects the
windows hidden after the change. A window a display change moves while it is
hidden stays hidden the way it was. A window the tiler did not hide, such as
one sent to another workspace and followed there, is shown with the
configured strategy. A window that leaves the layout, whether it closes, is
let go by the rules or is taken out for any other reason, is forgotten, so a
new window that gets its id is hidden the configured way.

When the window rules are reloaded, a hidden window is judged by the rules
alone: the platform may not count it as a candidate while it is out of view
(Windows rules out cloaked windows), so the tiler asks `Desktop::facts`
instead of `Desktop::candidate`. If the new rules let go of it, it is shown
again the way it was hidden rather than left out of view with nothing to
bring it back.

On Windows every strategy is available:

| Strategy | Method | Events |
|----------|--------|--------|
| `"cloak"` | COM ImmersiveShell cloaking | None |
| `"hide"` | `ShowWindow(SW_HIDE)`, shown with `SW_SHOWNOACTIVATE` | `EVENT_OBJECT_HIDE` |
| `"minimize"` | `ShowWindow(SW_MINIMIZE)`, shown with `SW_SHOWNOACTIVATE` | `EVENT_SYSTEM_MINIMIZESTART`, and `EVENT_SYSTEM_MINIMIZEEND` when shown |

**Cloak** is the recommended default. It uses the same undocumented COM
interface (`IApplicationView::SetCloak`) that Windows uses internally for
virtual desktops. Windows become invisible but keep their taskbar icons and
do not fire any events that would confuse the tiling manager.

The implementation lives in `crates/mosaico-windows/src/com/` and uses raw
`#[repr(C)]` vtable structs for the ImmersiveShell COM interfaces.

Note: `DwmSetWindowAttribute(DWMWA_CLOAK=13)` returns `E_ACCESSDENIED` from
user-mode processes. Only the COM approach works.

### Hidden Window Tracking

The events a switch causes look exactly like an application hiding or
restoring a window itself, which would make the tiler stop managing the
window or focus it. `tiler::Expectations` (`crates/mosaico-core/src/tiler/echo.rs`)
records every hide and show the applier carries out, with the `HidingEvents`
of the strategy used, and drops the matching events before the tiler sees
them:

- a `Hidden` or `Minimized` event for a window hidden a way that reports one,
  until the window is shown again;
- one `Restored` event for a window shown again a way that reports one, within
  `FOCUS_ECHO_TTL` of the show;
- a `Focused` event for a window hidden less than `HIDE_FOCUS_ECHO_TTL` ago,
  whatever the strategy.

A hide event for a window hidden a way that reports nothing, such as cloaking
on Windows, comes from the application itself and reaches the tiler. Showing a
hidden window can also report it as `Created`, which the tiler ignores for a
window it already manages.

### Cloaked Window Focus Handling

When using Cloak mode, cloaked windows retain their taskbar icons. If a user
clicks a cloaked window's taskbar icon, Windows fires `EVENT_SYSTEM_FOREGROUND`.
The `Focused` event handler detects that the window belongs to an inactive
workspace and automatically switches to that workspace.

### After the Daemon Is Killed

Hidden workspace windows are shown again on a clean `mosaico stop`. If the
daemon is killed or crashes instead, the next daemon shows them again on
startup from the record kept in the instance's data directory, whatever
`hiding` mode hid them, and then tiles them like any other window. See the
"Unclean exit" section in [daemon.md](daemon.md).

## Sending Windows

`send_to_workspace(n)` (in `crates/mosaico-core/src/tiler/workspace.rs`):

1. Removes the focused window from the current workspace
2. Adds it to workspace N on the same monitor
3. Hides the source workspace's remaining windows
4. Sets `active_workspace = n` on the focused monitor
5. Shows the target workspace's windows and focuses the moved window
6. In `Global` mode, calls `goto_workspace_on` for every other monitor
   (with `refocus=false`) so all monitors converge on workspace N

The window itself never crosses monitors. It stays on the monitor it was
on; global mode just keeps the *view* coherent across displays.

## Workspace Initialization

At daemon startup, all 8 workspaces are created per monitor (as empty
`Workspace` instances). Existing visible windows are added to workspace 1
(index 0) of their respective monitors. Only workspace 1 is active initially.

## Bar Integration

The status bar displays workspace indicators showing which workspace is
active. The `BarState` struct (`mosaico_core::bar`) provides:

- `active_workspace` -- 0-indexed active workspace
- `workspace_count` -- number of workspaces on the monitor

The workspaces widget renders every workspace number inside one container
pill, with the active workspace highlighted in the configured
`active_workspace` color.

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
- **Echo suppression follows what the platform reports.** The platform says
  which hide and restore events each hiding strategy causes, and those are
  taken for the tiler's own, so programmatic hides are not mistaken for
  windows closing. Focus reports for a window hidden moments ago are the one
  thing dropped whatever the strategy: hiding the focused window makes the OS
  report it for a while after the switch.
- **Per-monitor by default, global as opt-in.** Per-monitor matches the
  mental model where each monitor is independent (you can flip one screen
  without disturbing the others). The `Global` mode is for users who
  prefer Windows-virtual-desktop semantics. The choice is one config field
  (`workspaces.mode`); window ownership is unchanged across modes.
- **Windows never migrate between monitors during a switch.** Each
  monitor owns its own per-workspace window list. Global mode only
  synchronizes the *active workspace index*, not window placement.
- **Workspace 1 is always the initial active workspace**. New windows
  created after startup go to the active workspace of the focused monitor.

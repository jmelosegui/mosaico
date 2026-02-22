# Tiling Layout

Mosaico uses a Binary Space Partitioning (BSP) algorithm to arrange windows on
each monitor. The layout system is split into platform-agnostic types in
`mosaico-core` and orchestration logic in `mosaico-windows`.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/layout.rs` | `Layout` trait and `BspLayout` implementation |
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered collection of window handles per monitor |
| `crates/mosaico-core/src/rect.rs` | `Rect` -- rectangle type with spatial helpers |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-windows/src/tiling/mod.rs` | `TilingManager`, `MonitorState` -- core orchestration, event/action dispatch |
| `crates/mosaico-windows/src/tiling/navigation.rs` | `focus_direction()`, `move_direction()` -- spatial navigation and cross-monitor moves |
| `crates/mosaico-windows/src/tiling/layout.rs` | `apply_layout_on()`, `retile_all()`, `toggle_monocle()` -- layout application |
| `crates/mosaico-windows/src/tiling/workspace.rs` | `goto_workspace()`, `send_to_workspace()` -- workspace switching |

### Key Types

- `Layout` (trait) -- defines `fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>`
- `BspLayout` -- implements `Layout` with configurable `gap` and `ratio`
- `Workspace` -- maintains an ordered `Vec<usize>` of window handles for a
  monitor; provides `add()`, `insert()`, `remove()`, `swap()`,
  `compute_layout()`
- `Rect` -- fields: `x`, `y`, `width`, `height`; methods: `center_x()`,
  `center_y()`, `vertical_overlap()`, `horizontal_overlap()`
- `TilingManager` -- central orchestrator holding `Vec<MonitorState>`, the
  layout instance, rules, border, focus state, `applying_layout` flag, and
  `hidden_by_switch: HashSet<usize>` for workspace tracking
- `MonitorState` -- per-monitor state: `id`, `work_area`,
  `workspaces: Vec<Workspace>` (up to 8), `active_workspace: usize`, `monocle`

## BSP Layout Algorithm

`BspLayout::split()` recursively divides the available area:

1. **One window** -- fills the entire area (minus gaps)
2. **Two or more windows** -- splits the area in half:
   - Even depth: **horizontal split** (left/right)
   - Odd depth: **vertical split** (top/bottom)
3. The first window always gets the larger partition (controlled by `ratio`,
   default 0.5)
4. Remaining windows are recursively laid out in the second partition

### Gap Handling

- Outer gap: applied around the entire work area
- Inner gap: half-gap spacing between adjacent windows
- Dimensions are clamped to `max(1)` to prevent zero/negative sizes when gaps
  exceed available space

### Configuration

- `gap: i32` -- pixel gap between windows (default: 8, range: 0-200)
- `ratio: f64` -- split ratio for the primary partition (default: 0.5,
  range: 0.1-0.9)

## TilingManager

The `TilingManager` is instantiated in the daemon loop and processes both
window events and user actions.

### Event Handling

`handle_event()` dispatches `WindowEvent` variants:

- `Created` -- adds window to the focused monitor's workspace if tileable
- `Destroyed` -- removes window from whichever workspace contains it
- `Focused` -- updates focus tracking, shows/hides border
- `Moved` -- detects manual monitor reassignment (drag to different monitor)
- `Minimized` -- removes window from workspace, re-tiles
- `Restored` -- re-adds window to workspace

### Action Handling

`handle_action()` dispatches `Action` variants to dedicated methods in
the split tiling modules:

- `Focus(dir)` -> `focus_direction(dir)` (in `navigation.rs`) -- spatial
  navigation. Left/Right use `resolve_horizontal_target()` and can cross
  monitors. Up/Down use `find_same_monitor_neighbor()` and stay on the
  current monitor.
- `Move(dir)` -> `move_direction(dir)` (in `navigation.rs`) -- spatial swap.
  Left/Right can move windows across monitors. Up/Down swap on the same
  monitor.
- `Retile` -> `retile_all()` (in `layout.rs`) -- re-applies layout on all
  monitors
- `ToggleMonocle` -> `toggle_monocle()` (in `layout.rs`) -- toggles monocle
  mode
- `CloseFocused` -> `close_focused()` -- sends `WM_CLOSE`
- `GoToWorkspace(n)` -> `goto_workspace(n)` (in `workspace.rs`) -- switches
  to workspace N on the focused monitor
- `SendToWorkspace(n)` -> `send_to_workspace(n)` (in `workspace.rs`) -- sends
  focused window to workspace N

### Config Reload

The `TilingManager` supports hot-reloading of configuration:

- `reload_config(config)` -- updates the `BspLayout` gap/ratio and
  `BorderConfig`, then retiles all windows and updates the border
- `reload_rules(rules)` -- replaces the window rules used for tileability
  checks on new windows
- `adjust_work_areas_for_bar(height, indices)` -- shrinks work areas on
  monitors that display a bar (offsets `y` and `height` by the bar height)
- `reset_and_adjust_work_areas(height, indices)` -- resets to original work
  areas and re-applies bar adjustment (used on bar config reload)
- `bar_states()` -- returns per-monitor `BarState` with workspace count,
  active workspace, active window title, etc.
- `restore_all_windows()` -- shows all windows across every workspace and
  monitor; called on shutdown so windows hidden by workspace switching are
  not left invisible

### Layout Application

`apply_layout_on(monitor_idx)`:

1. Prunes stale handles (windows that closed without firing destroy events)
2. In **monocle mode**: positions only the focused window to fill the work area
3. In **normal mode**: calls `BspLayout::apply()` to compute positions, then
   calls `set_rect()` + `invalidate()` on each window

The `applying_layout` flag suppresses `Moved` events during programmatic layout
to prevent re-entrant tiling.

### Window Tileability

`is_tileable(hwnd)` checks:

1. Window is visible
2. Window has `WS_CAPTION` and is not `WS_EX_TOOLWINDOW`
3. Window passes `should_manage()` rule evaluation

## Monocle Mode

A per-monitor toggle that makes the focused window fill the entire work area
(minus gaps). Other windows remain in the workspace but are not repositioned.

- Toggled via `Alt+T` (default) or `mosaico action toggle-monocle`
- The border color changes to the configured `monocle` color when active
- Tracked by the `monocle: bool` field on `MonitorState`

When monocle is active, `apply_layout_on()` skips BSP computation and directly
positions the focused window to fill the monitor's work area with gap padding.

## Workspace Management

Each monitor has up to 8 workspaces (`MAX_WORKSPACES = 8`), stored as a
`Vec<Workspace>` in `MonitorState`. One workspace per monitor is active at a
time (`active_workspace` index).

The `Workspace` type maintains window ordering:

- `add(handle)` -- appends to the end (no duplicates)
- `insert(index, handle)` -- inserts at a specific position (index clamped
  to length, no duplicates). Used when moving windows across monitors to
  control BSP slot placement.
- `remove(handle)` -- removes and shifts remaining handles
- `swap(a, b)` -- swaps two handles by index
- `compute_layout(layout, work_area)` -- delegates to `Layout::apply()`
- `handles()` -- returns a slice of all window handles in the workspace

New windows are always placed on the **focused** monitor's **active
workspace**, not the OS-determined monitor. When moving a window to an
adjacent monitor, `insert(0, hwnd)` is used when entering from the right
(leftmost BSP slot) and `add(hwnd)` when entering from the left.

### Workspace Switching

`goto_workspace(n)` (in `workspace.rs`):

1. If already on workspace N, do nothing
2. Hide all windows in the current workspace via `Window::hide()`
3. Record hidden handles in `hidden_by_switch` so that the `Hide` event
   handler does not remove them from the workspace
4. Show all windows in the target workspace via `Window::show()`
5. Update `active_workspace`, retile, and focus the first window

`send_to_workspace(n)` (in `workspace.rs`):

1. Remove the focused window from the current workspace
2. Add it to the target workspace
3. Hide the window (it is now on a non-active workspace)
4. Record the handle in `hidden_by_switch`
5. Retile and focus the next window on the current workspace

The `hidden_by_switch` set prevents workspace-hidden windows from being
treated as destroyed when the `EVENT_OBJECT_HIDE` event fires.

## Design Decisions

- BSP alternates horizontal/vertical splits by recursion depth, producing a
  balanced grid-like layout.
- The first window always gets the primary partition, making the "main" window
  consistently positioned.
- `applying_layout` flag prevents infinite loops from programmatic window
  moves triggering `Moved` events.
- Stale handle pruning runs before every layout application to handle UWP apps
  that close without firing destroy events.
- `Workspace` is intentionally simple (just a `Vec<usize>`) -- the layout
  algorithm determines spatial positioning, not insertion order.
- The tiling module is split into four files (`mod.rs`, `navigation.rs`,
  `layout.rs`, `workspace.rs`) to keep each file under ~150 lines with one
  concern per file.
- `hidden_by_switch` is a `HashSet<usize>` on the `TilingManager` (not per
  workspace) because the event handler needs to check all hidden windows
  regardless of which workspace they belong to.
- `restore_all_windows()` iterates all workspaces on all monitors to ensure
  no window remains invisible after daemon shutdown.

## Tests

- 5 unit tests for `BspLayout`: single window, two windows, three windows,
  empty input, large gap safety
- 3 unit tests for `Workspace`: add/remove, insert at position, layout
  delegation
- 13 unit tests in `tiling/mod.rs`: event handling, action dispatch,
  workspace switching, config reload, bar state, window restore
- Extensive spatial navigation tests in `spatial.rs` (see
  [monitor-management.md](monitor-management.md))

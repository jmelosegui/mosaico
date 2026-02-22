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
| `crates/mosaico-windows/src/tiling.rs` | `TilingManager` and `MonitorState` -- orchestrates layout across monitors |

### Key Types

- `Layout` (trait) -- defines `fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>`
- `BspLayout` -- implements `Layout` with configurable `gap` and `ratio`
- `Workspace` -- maintains an ordered `Vec<usize>` of window handles for a
  monitor; provides `add()`, `insert()`, `remove()`, `swap()`,
  `compute_layout()`
- `Rect` -- fields: `x`, `y`, `width`, `height`; methods: `center_x()`,
  `center_y()`, `vertical_overlap()`, `horizontal_overlap()`
- `TilingManager` -- central orchestrator holding `Vec<MonitorState>`, the
  layout instance, rules, border, and focus state
- `MonitorState` -- per-monitor state: `id`, `work_area`, `workspace`, `monocle`

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

`handle_action()` dispatches `Action` variants to dedicated methods:

- `Focus(dir)` -> `focus_direction(dir)` -- spatial navigation in any
  direction. Left/Right use `resolve_horizontal_target()` and can cross
  monitors. Up/Down use `find_same_monitor_neighbor()` and stay on the
  current monitor.
- `Move(dir)` -> `move_direction(dir)` -- spatial swap in any direction.
  Left/Right can move windows across monitors. Up/Down swap on the same
  monitor.
- `Retile` -> `retile_all()` -- re-applies layout on all monitors
- `ToggleMonocle` -> `toggle_monocle()` -- toggles monocle mode
- `CloseFocused` -> `close_focused()` -- sends `WM_CLOSE`

### Config Reload

The `TilingManager` supports hot-reloading of configuration:

- `reload_config(config)` -- updates the `BspLayout` gap/ratio and
  `BorderConfig`, then retiles all windows and updates the border
- `reload_rules(rules)` -- replaces the window rules used for tileability
  checks on new windows

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

The `Workspace` type maintains window ordering:

- `add(handle)` -- appends to the end (no duplicates)
- `insert(index, handle)` -- inserts at a specific position (index clamped
  to length, no duplicates). Used when moving windows across monitors to
  control BSP slot placement.
- `remove(handle)` -- removes and shifts remaining handles
- `swap(a, b)` -- swaps two handles by index
- `compute_layout(layout, work_area)` -- delegates to `Layout::apply()`

New windows are always placed on the **focused** monitor, not the OS-determined
monitor. When moving a window to an adjacent monitor, `insert(0, hwnd)` is
used when entering from the right (leftmost BSP slot) and `add(hwnd)` when
entering from the left.

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

## Tests

- 5 unit tests for `BspLayout`: single window, two windows, three windows,
  empty input, large gap safety
- 3 unit tests for `Workspace`: add/remove, insert at position, layout
  delegation
- Extensive spatial navigation tests in `spatial.rs` (see
  [monitor-management.md](monitor-management.md))

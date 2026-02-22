# Monitor Management & Spatial Navigation

Mosaico enumerates all connected monitors, queries their work areas (excluding
the taskbar), and provides four-directional spatial navigation across windows
and monitors using vim-style H/J/K/L motions.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/spatial.rs` | Pure spatial functions: `find_neighbor()`, `find_entry()` |
| `crates/mosaico-core/src/rect.rs` | `Rect::vertical_overlap()`, `Rect::horizontal_overlap()` |
| `crates/mosaico-windows/src/monitor.rs` | `MonitorInfo`, enumeration and query functions |
| `crates/mosaico-windows/src/tiling/mod.rs` | `TilingManager`, `MonitorState`, `SpatialTarget` |
| `crates/mosaico-windows/src/tiling/navigation.rs` | Multi-monitor focus/move logic, spatial target resolution |

### Key Types

- `MonitorInfo` -- fields: `id: usize` (HMONITOR cast), `work_area: Rect`
- `SpatialTarget` (enum) -- `Neighbor(hwnd)` or `AdjacentMonitor(idx)`
- `Direction` (enum) -- `Left`, `Right`, `Up`, `Down` (from `mosaico-core`)

## Monitor Enumeration

`enumerate_monitors()` uses `EnumDisplayMonitors` to discover all connected
monitors, returning them sorted left-to-right by `work_area.x`.

Additional utility function:

- `monitor_id_for_window(hwnd)` -- returns the HMONITOR id for a window's
  monitor

## Spatial Navigation

All four directions (Left, Right, Up, Down) use BSP-aware spatial neighbor
lookup based on computed layout positions rather than workspace insertion order.

### Spatial Module (`mosaico-core/src/spatial.rs`)

The spatial navigation logic lives in `mosaico-core` as pure functions over
`(handle, Rect)` slices, making them unit-testable without Win32 dependencies.

#### `find_neighbor(positions, focused, direction)`

The core spatial algorithm:

1. **Filter by direction**: keeps candidates whose center is beyond `focused`
   in the requested direction (e.g., `center_x() > focused.center_x()` for
   Right)
2. **Filter by perpendicular overlap**: for horizontal directions, requires
   `vertical_overlap > 0`; for vertical, requires `horizontal_overlap > 0`.
   This prevents jumping to diagonally-adjacent windows.
3. **Pick nearest**: uses edge distance (gap between touching edges) as the
   primary key, with perpendicular center as tiebreaker (topmost for
   horizontal directions, leftmost for vertical)

#### `find_entry(positions, direction)`

Finds the best window to focus when entering a monitor from a given direction.
Picks the topmost window first, breaking ties by the edge closest to the
direction of travel (leftmost when entering from the left, rightmost when
entering from the right).

### Horizontal vs Vertical Behavior

- **Left/Right**: spatial horizontal neighbor lookup on the current monitor.
  If no neighbor exists, overflows to the physically adjacent monitor (no
  wrapping -- stops at the leftmost/rightmost monitor).
- **Up/Down**: spatial vertical neighbor lookup on the current monitor only.
  Stops at the boundary (no cross-monitor overflow).

### Resolution Strategy

`resolve_horizontal_target(direction)` (for Left/Right) in `navigation.rs`:

1. Computes BSP layout positions for all windows on the current monitor
2. Calls `spatial::find_neighbor()` to look for a same-monitor neighbor
3. If a neighbor is found: returns `SpatialTarget::Neighbor(hwnd)`
4. If no neighbor exists: calls `find_adjacent_monitor(direction)` to find
   the nearest monitor in the requested direction by `center_x` comparison.
   Returns `SpatialTarget::AdjacentMonitor(idx)` or `None` if at the edge.

`find_adjacent_monitor(direction)` scans all monitors and picks the one
closest in the given horizontal direction based on work area center_x values.
There is no wrapping -- navigation stops at the leftmost/rightmost monitor.

`find_same_monitor_neighbor(direction)` (for Up/Down) in `navigation.rs`:

1. Computes BSP layout positions
2. Calls `spatial::find_neighbor()` for vertical lookup
3. Returns `Some(hwnd)` or `None` at boundary

## Focus Actions

`focus_direction(dir)`:

- **Left/Right**: resolves horizontal target. If `Neighbor(hwnd)`, focuses
  that window. If `AdjacentMonitor(idx)`, uses `find_entry()` to pick the
  best entry window on the target monitor.
- **Up/Down**: finds same-monitor neighbor. If found, focuses it. Otherwise
  does nothing (stops at boundary).

## Move Actions

`move_direction(dir)`:

- **Left/Right**: resolves horizontal target. If `Neighbor(hwnd)`, swaps the
  two windows in the workspace and retiles. If `AdjacentMonitor(idx)`,
  removes the window from the source workspace and inserts it into the target
  workspace (position depends on direction), then retiles both monitors.
- **Up/Down**: finds same-monitor neighbor. If found, swaps the two windows
  by workspace index and retiles.

### Cross-Monitor Insertion

When moving a window to an adjacent monitor:
- Moving **right**: inserts at position 0 (leftmost BSP slot)
- Moving **left**: appends to end (rightmost BSP slot)

This places the window at the entry side of the target monitor.

## Monitor Reassignment

`reassign_monitor(hwnd)` handles manual window drags between monitors:

1. Called when a `Moved` event is received
2. Queries the window's current monitor via `MonitorFromWindow`
3. If the monitor differs from the workspace's monitor, moves the window
   between workspaces and re-tiles both monitors

## Daemon Initialization

At startup, `daemon_loop()`:

1. Calls `enumerate_monitors()` to get all monitor info
2. Creates a `MonitorState` for each monitor with an empty `Workspace`
3. Enumerates existing visible windows and assigns them to the correct
   monitor's workspace
4. Applies the initial layout on all monitors

## Design Decisions

- Monitors sorted left-to-right provides intuitive H/L navigation matching
  physical monitor placement.
- `SpatialTarget` enum cleanly separates "same monitor neighbor" from
  "cross-monitor jump", allowing the same logic to handle both focus and
  move operations.
- Spatial navigation lives in `mosaico-core` as pure functions, keeping the
  core crate testable and the platform crate focused on Win32 integration.
- `find_adjacent_monitor()` finds the physically nearest monitor by
  `center_x` comparison rather than using a fixed index offset, handling
  non-uniform monitor arrangements correctly.
- **No wrapping**: Left/Right stop at the edge of the monitor array rather
  than wrapping around. This is more intuitive for physical monitor layouts.
- Vertical overlap (for horizontal navigation) and horizontal overlap (for
  vertical navigation) prevent unintuitive diagonal jumps.
- Edge distance as the primary sort key ensures the nearest window wins,
  with perpendicular center as tiebreaker for consistent ordering.

## Tests

The `spatial.rs` module has extensive unit tests using predefined BSP layouts
(2, 3, 4, and 5 window configurations):

- Horizontal navigation: right-from-left, left-from-right, topmost tiebreaker,
  immediate neighbor preference over distant ones
- Vertical navigation: up/down between BSP quadrants, correct tiebreaking
- Boundary tests: no neighbor at edges (left/right/up/down), single window
- Entry tests: `find_entry` from left/right for various layouts

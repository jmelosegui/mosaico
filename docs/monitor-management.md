# Monitor Management & Spatial Navigation

Mosaico enumerates all connected monitors, queries their work areas (excluding
the taskbar), and provides spatial sorting for multi-monitor navigation using
vim-style directional motions.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-windows/src/monitor.rs` | `MonitorInfo`, enumeration and query functions |
| `crates/mosaico-windows/src/tiling.rs` | Multi-monitor focus/move logic, spatial navigation |
| `crates/mosaico-core/src/rect.rs` | `Rect::vertical_overlap()` for spatial neighbor detection |

### Key Types

- `MonitorInfo` -- fields: `id: usize` (HMONITOR cast), `work_area: Rect`
- `SpatialTarget` (enum) -- `Neighbor(hwnd)` or `AdjacentMonitor(idx)`

## Monitor Enumeration

`enumerate_monitors()` uses `EnumDisplayMonitors` to discover all connected
monitors, returning them sorted left-to-right by `work_area.x`. This
ordering means:

- "Next" = right (direction `+1`)
- "Previous" = left (direction `-1`)

Additional utility functions:

- `primary_work_area()` -- returns the primary monitor's work area
- `work_area_for_window(hwnd)` -- returns the work area of the monitor
  containing a specific window
- `monitor_id_for_window(hwnd)` -- returns the HMONITOR id for a window's
  monitor

## Spatial Navigation

Focus and move operations for H/L keys use BSP-aware spatial neighbor lookup
rather than simple list-order cycling.

### Resolution Strategy

`resolve_spatial_target(direction)`:

1. Computes BSP layout positions for all windows on the current monitor
2. Calls `find_spatial_neighbor()` to look for a window in the requested
   direction
3. If a neighbor is found: returns `SpatialTarget::Neighbor(hwnd)`
4. If no neighbor exists: returns `SpatialTarget::AdjacentMonitor(idx)`,
   wrapping with `rem_euclid` for circular navigation

### Spatial Neighbor Algorithm

`find_spatial_neighbor(positions, focused_rect, direction)`:

1. Filters candidates strictly in the requested direction:
   - Direction `+1` (right): candidate center X > focused center X
   - Direction `-1` (left): candidate center X < focused center X
2. Requires positive **vertical overlap** between the candidate and focused
   window (prevents jumping to diagonally-adjacent windows)
3. Selects the best match by:
   - Maximum vertical overlap (prefers windows most aligned vertically)
   - Minimum horizontal distance (tiebreaker)

## Focus Monitor Actions

`focus_monitor(direction)`:

1. Resolves the spatial target
2. If `Neighbor(hwnd)`: calls `set_foreground()` on the neighbor window
3. If `AdjacentMonitor(idx)`: focuses the first window (direction > 0) or
   last window (direction < 0) in the target monitor's workspace; updates
   `focused_monitor` tracking

## Move to Monitor Actions

`move_to_monitor(direction)`:

1. Resolves the spatial target
2. If `Neighbor(hwnd)`: swaps the focused window with the neighbor in the
   workspace order, then re-tiles
3. If `AdjacentMonitor(idx)`:
   - Removes the window from the source monitor's workspace
   - Adds it to the target monitor's workspace
   - Re-tiles both monitors
   - Focuses the moved window on the target monitor

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
- `find_spatial_neighbor()` uses BSP layout positions (computed rectangles)
  rather than workspace insertion order, providing truly spatial left/right
  navigation.
- Vertical overlap requirement prevents unintuitive diagonal jumps.
- Monitor indices wrap around via `rem_euclid` for circular navigation (last
  monitor -> first monitor).
- When moving to an adjacent monitor, focus targets the first or last window
  depending on direction, maintaining spatial continuity.

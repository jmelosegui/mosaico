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
| `crates/mosaico-core/src/monitor.rs` | `MonitorKey`, `EdidIdentity`, `Monitor` trait |
| `crates/mosaico-core/src/monitor/matching.rs` | `match_monitors()` |
| `crates/mosaico-core/src/monitor/edid.rs` | `EdidIdentity::from_edid()`, `pnp_manufacturer()` |
| `crates/mosaico-windows/src/monitor.rs` | `MonitorInfo`, enumeration and query functions |
| `crates/mosaico-windows/src/monitor_identity.rs` | Builds a `MonitorKey` from the display configuration |
| `crates/mosaico-windows/src/monitor_edid.rs` | Reads the EDID copy from the registry |
| `crates/mosaico-windows/src/tiling/display.rs` | `handle_display_change()` |
| `crates/mosaico-windows/src/tiling/display_rebuild.rs` | `rebuild_monitors()`, `pair_monitors()` (no Win32 calls, unit tested) |
| `crates/mosaico-windows/src/tiling/mod.rs` | `TilingManager`, `MonitorState`, `SpatialTarget` |
| `crates/mosaico-windows/src/tiling/navigation.rs` | Multi-monitor focus/move logic, spatial target resolution |

### Key Types

- `MonitorInfo` -- fields: `id: usize` (HMONITOR cast), `key: MonitorKey`,
  `work_area: Rect`, `primary: bool`. Implements `mosaico_core::Monitor`
- `MonitorKey` -- stable identity of a physical display (from `mosaico-core`)
- `SpatialTarget` (enum) -- `Neighbor(hwnd)` or `AdjacentMonitor(idx)`
- `Direction` (enum) -- `Left`, `Right`, `Up`, `Down` (from `mosaico-core`)

## Monitor Enumeration

`enumerate_monitors()` uses `EnumDisplayMonitors` to discover all connected
monitors, returning them sorted left-to-right by `work_area.x`. Each one
gets a `MonitorKey` from `monitor_identity::display_identities()`.

Additional utility function:

- `monitor_id_for_window(hwnd)` -- returns the HMONITOR id for a window's
  monitor

## Monitor Identity

Windows hands out new HMONITOR values every time the display configuration
changes, including for monitors that stayed connected. The id is therefore
only good for runtime lookups (`MonitorFromWindow`). To recognise a monitor
after a reconnect or a KVM switch, each monitor carries a `MonitorKey`.

### The contract (`mosaico-core/src/monitor.rs`)

The key and its matching rules live in the core crate so every platform
restores layouts the same way. A platform implements the `Monitor` trait:

```rust
pub trait Monitor {
    fn key(&self) -> &MonitorKey;
    fn work_area(&self) -> Rect;
    fn is_primary(&self) -> bool;
}
```

`MonitorKey` has three optional fields:

| Field | Meaning |
|-------|---------|
| `stable_id` | Most specific stable identifier the platform offers. Opaque, compared for equality |
| `edid` | `EdidIdentity { manufacturer, product, serial }` decoded from the display's EDID. Means the same on every OS |
| `connector` | Port the display is attached through. Opaque, compared for equality |

Rules a platform implementation must follow:

- The key stays the same across unplug and replug, KVM switches, sleep,
  reboots, and resolution, scaling or arrangement changes.
- It is never built from per-session handles, enumeration order, desktop
  position or resolution.
- Two displays of the same model on different ports differ in at least one
  field when the hardware allows it.
- A field the platform cannot read is `None`. A wrong value causes false
  matches; a missing one only weakens matching.

What each platform is expected to supply:

| Field | Windows | macOS | Linux |
|-------|---------|-------|-------|
| `stable_id` | Monitor device interface path (`DisplayConfigGetDeviceInfo`, target name) | `CGDisplayCreateUUIDFromDisplayID` | Connector name plus an EDID hash |
| `edid` | The EDID copy under `Enum\DISPLAY` in the registry, decoded by `EdidIdentity::from_edid()`; falls back to `edidManufactureId` / `edidProductCodeId` without a serial | `CGDisplayVendorNumber` / `ModelNumber` / `SerialNumber` | Raw EDID from the RandR property or DRM sysfs, decoded by `EdidIdentity::from_edid()` |
| `connector` | Output technology and `connectorInstance`, e.g. `dp:1` | none | Connector name, e.g. `DP-1` |

### Matching

`match_monitors(old, new)` pairs keys one to one, running one tier at a
time so a weak match can never take a display a stronger one would claim:

1. Same `stable_id`
2. Same EDID including a known serial
3. Same EDID model on the same `connector` (tells identical monitors
   without serials apart)
4. Same EDID model, when it is the only one of that model on both sides

Two known serials that differ never match. A key with every field `None`
never matches anything.

`pair_monitors()` in the Windows tiler runs `match_monitors` first. Only
monitors whose key is unknown fall back to the same HMONITOR and then to
the same work area origin. A monitor with a known key is never paired with
another by position, so an office monitor that sits where the home monitor
was does not inherit the home monitor's workspaces.

`mosaico debug list` prints each monitor's key.

## Disconnect and Reconnect

`handle_display_change()` calls `rebuild_monitors()` (`display_rebuild.rs`),
which keeps a `ReconnectMemory` (`tiling/reconnect.rs`) owned by the
`TilingManager`:

- **Paired monitors** keep their workspaces and active workspace.
- **Removed monitors** are passed to `ReconnectMemory::park()`. Each window
  moves to the same workspace index on the primary monitor (the host), and
  a `Displaced { home, workspace, slot, host, host_workspace }` record is
  kept for it. The monitor's workspaces, now empty but still carrying
  their layout, monocle and last focused window, are parked under its key
  with its active workspace.
- **Returning monitors** (unpaired, but with a parked key) get their parked
  workspaces back through `unpark()`. `restore()` then moves each window
  whose home is that monitor back to its workspace, in slot order, but only
  if it is still on the host workspace it was put on. A window the user
  moved somewhere else stays there. The record is dropped either way.
  Monocle is turned off on a workspace whose monocle window did not return.
- **New monitors** start with workspaces built from `[workspaces.layouts]`.

A window displaced twice (its host disconnects as well) keeps its original
home and only follows the new host. Monitors with an unknown key are never
parked and their windows get no record, because they could not be
recognised on return. `Destroyed` events call `ReconnectMemory::forget()`
so a reused hwnd is never sent anywhere.

`rebuild_monitors()` returns the windows that moved. Those that landed on
an inactive workspace are hidden before the retile and those on an active
workspace are shown after it. Focus stays on the same physical monitor if
it is still connected, otherwise it moves to the primary monitor.

Display changes are applied once the configuration settles. The daemon
loop does not rebuild on each `DisplayChanged` event: it records it in a
`DisplaySettle` (`daemon_settle.rs`) and rebuilds once no further change
has arrived for 750 ms. A KVM switch or a dock sends several
`WM_DISPLAYCHANGE` messages in a row, and rebuilding on each one would
retile every window several times and could park a monitor that is only
renegotiating its signal.

The memory is not persisted: it survives sleep but not a daemon restart.
`mosaico debug state` lists the parked monitors and the number of
displaced windows.

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

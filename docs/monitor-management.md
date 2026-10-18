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
| `crates/mosaico-core/src/tiler/enumerator.rs` | `Enumerator`, the contract for listing monitors and windows, and the left-to-right order |
| `crates/mosaico-windows/src/enumerate.rs` | `WindowsEnumerator`, the Windows `Enumerator` |
| `crates/mosaico-windows/src/monitor.rs` | Monitor enumeration and query functions |
| `crates/mosaico-windows/src/monitor_identity.rs` | Builds a `MonitorKey` from the display configuration |
| `crates/mosaico-windows/src/monitor_edid.rs` | Reads the EDID copy from the registry |
| `crates/mosaico-core/src/tiler/display.rs` | `handle_display_change()` |
| `crates/mosaico-core/src/tiler/display_rebuild.rs` | `rebuild_monitors()`, `pair_monitors()` (unit tested) |
| `crates/mosaico-core/src/tiler/display_monocle.rs` | `join_monocle()`, `clear_stale_monocle()`: monocle workspaces across a display change, and the windows that wait behind their monocle window |
| `crates/mosaico-core/src/tiler/state.rs` | `MonitorState`, `SpatialTarget` |
| `crates/mosaico-core/src/tiler/navigation.rs`, `navigation_helpers.rs` | Multi-monitor focus/move logic, spatial target resolution |
| `crates/mosaico-core/src/tiler/manager_inputs.rs` | `TilingManager`, which feeds the tiler the monitor lists |

### Key Types

- `Display` -- a connected monitor (from `mosaico-core`): `id: MonitorId`
  (the HMONITOR on Windows), `key: MonitorKey`, `work_area: Rect`,
  `primary: bool`. Implements `mosaico_core::Monitor`
- `MonitorKey` -- stable identity of a physical display (from `mosaico-core`)
- `SpatialTarget` (enum) -- `Neighbor(window)` or `AdjacentMonitor(idx)`
- `Direction` (enum) -- `Left`, `Right`, `Up`, `Down` (from `mosaico-core`)

## Monitor Enumeration

Monitors are listed through the core `Enumerator` contract
(`mosaico-core/src/tiler/enumerator.rs`). A platform implements
`connected_monitors()`, which returns every monitor of the desktop in any
order: mirrored monitors count once, exactly one is primary, and work areas
leave out taskbars and docks, in the same coordinates as window bounds.

Callers use the provided `monitors()`, which sorts that list in core by the
left edge of the work area. The sort is stable, so monitors that share a left
edge (stacked vertically) stay in the order the platform found them in. This
order is what a monitor number means everywhere: the bar's `monitors`
setting, `mosaico debug list`, the tiler's monitor indices and the monitor
that has focus at startup. The bar and the tiler both number monitors from
this list.

On Windows, `WindowsEnumerator` gets the monitors from `EnumDisplayMonitors`
(`monitor::enumerate_monitors()`), and each one gets a `MonitorKey` from
`monitor_identity::display_identities()`.

Which monitor a window is on is a per-window query the tiler makes while it
decides, so it lives on `Desktop::monitor_of`: the monitor that holds the
largest part of the window, the one a minimized window will be restored to,
or the primary monitor for a window on none. On Windows it is
`MonitorFromWindow` (`monitor::monitor_id_for_window()`).

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

`pair_monitors()` in the core tiler runs `match_monitors` first. Only
monitors whose key is unknown fall back to the same HMONITOR and then to
the same work area origin. A monitor with a known key is never paired with
another by position, so an office monitor that sits where the home monitor
was does not inherit the home monitor's workspaces.

`mosaico debug list` prints each monitor's key.

## Disconnect and Reconnect

`handle_display_change()` calls `rebuild_monitors()` (`display_rebuild.rs`),
which keeps a `ReconnectMemory` (`tiler/reconnect.rs`) owned by the core
`Tiler`:

- **Paired monitors** keep their workspaces and active workspace.
- **Removed monitors** are passed to `ReconnectMemory::park()`. Each window
  moves to the same workspace index on the primary monitor (the host), and
  a `Displaced { home, workspace, slot, splits, host, host_workspace }`
  record is kept for it, `splits` being what it owned there, read before
  any window leaves: its split of the BSP chain where the user moved it,
  and its share of the stack and of its side column where the user resized
  them. The monitor's workspaces, now empty but still carrying their
  layout, monocle, last focused window and the masters' ratios, are parked
  under its key with its active workspace.
- **Returning monitors** (unpaired, but with a parked key) get their parked
  workspaces back through `unpark()`. `restore()` then moves each window
  whose home is that monitor back to its workspace, in slot order, but only
  if it is still on the host workspace it was put on, and once they are all
  back, gives each its split and its shares again at its place there,
  whichever layout is on screen: in each column, the windows back keep
  their shares and the others share what is left, so a whole monitor back
  in order has the heights of its stacks back, but no share of the stack
  goes to a window back as the master, and a window back in the other side
  column sends both columns back to equal heights, as the same close at home
  does. A window the user moved somewhere else stays there. The record is
  dropped either way.
  A returning monitor's workspace whose monocle window did not come back
  leaves monocle. A workspace whose monocle window went home is settled once
  the windows that moved have joined it: it stays in monocle when windows wait
  behind it there, with the one that has focus, or else the one in front of
  them, as its monocle window. Otherwise it keeps monocle with no monocle
  window if it is empty, and leaves monocle if other windows remain. A stale
  monocle anywhere else, such as one whose monocle window the user sent away,
  is left for that window to come back to.
- **New monitors** start with workspaces built from `[workspaces.layouts]`.

A window displaced twice (its host disconnects as well) keeps its original
home and only follows the new host.

Keys can change for a monitor that stays connected. While a KVM is switched
away, Windows may keep an emulated stand-in for one monitor that reports the
EDID model but no serial or device path, and the full key comes back when
the KVM returns. So identity is followed through pairing, never compared
exactly across a change:

- `rehost()` rewrites the host of every displaced window to the new key of
  the monitor it was paired with, in one pass so renames cannot chain.
- `unpark()` finds the parked state with `match_monitors()`, so a monitor
  that returns with a reduced key still gets its workspaces back.
  `restore()` then matches windows against the key the monitor was parked
  under.
- A host with an unknown key still keeps the records; pairing by HMONITOR or
  position follows it. Monitors with an unknown key are never
parked and their windows get no record, because they could not be
recognised on return. `Destroyed` events call `ReconnectMemory::forget()`
so a reused window id is never sent anywhere.

`rebuild_monitors()` returns the windows that moved. A window parked off a
monitor that left and sent back home to one that returned in the same
change is listed twice, but is hidden or shown once. Focus settles first,
as described below. Those that landed on an inactive workspace are hidden
before the retile. Those on an active workspace are shown after it, so
they appear in their final place; the retile keeps a window the tiler hid
with `hide` while it is still hidden. A window the tiler minimized, and
that is still minimized, is restored before the retile instead, as a
workspace switch does, because its slot cannot be set until it is
restored. All of them go through `HiddenHow`
(`crates/mosaico-core/src/tiler/hidden_how.rs`): a window is shown the way
the tiler hid it, and one that is already hidden stays hidden the way it
was rather than being hidden again with the configured strategy (see
[workspaces.md](workspaces.md#hiding-behaviour)). A window that lands on a
monocle workspace waits behind its monocle window, with focus or not (see
[tiling-layout.md](tiling-layout.md#monocle-mode)).

A focused window that is still in view keeps focus, wherever it moved,
unless the change has it wait behind a monocle window: then the monocle
window takes focus, as it is the one on screen. Otherwise focus stays on the
same physical monitor if it is still connected, followed through its
pairing, so a monitor that reports a different key keeps focus. When that
monitor is gone, focus moves to the primary monitor, which took in its
windows. If the focused window ends up out of view, the focused monitor's
window takes focus as in a workspace switch: its monocle window, then the
one it remembers, then its first. With nothing there, focus is parked on
that monitor until something takes it. Either way the system's own pick
once the window is hidden is not followed: the echo filter drops it while
the focus change is held, and the parked claim while focus is parked. When
the pointer follows focus, it moves onto the window that takes focus.

In global mode the monitors are always in step: every switch moves all of
them, and a reload that turns global mode on lines them up. A display
change keeps the workspace that was in view before it. Monitors that
connect, new or returning, switch to it, even when the monitor that had
focus is the one that went away, and a returning monitor's windows move
back with it. As a safeguard, a monitor that is out of step anyway is
switched too: the windows of the workspace it leaves are hidden and those
of the one it switches to tiled and shown, along with the windows that
moved, and a warning is logged.

After the retile, `refresh_visible_windows()` calls `redraw()` on every
window of an active workspace, including those the layout skipped because
they were already in place; a user retile does the same. On Windows,
`redraw()` also repairs Chromium windows (`chromium.rs`): Chromium
presents through an `Intermediate D3D Window` child owned by its GPU
process, which can be left at a stale offset after the window moves across
displays with different DPI, such as the emulated display a KVM leaves
behind. Input still lands in the right place but the picture is drawn
shifted, and Chromium only resizes that child afterwards, never moves it,
so the child is moved back to the client origin. A display change that
changes anything, and a user retile, also draw every border from scratch,
rather than moving the bitmap it kept, as
[Window Border Overlays](focus-border.md) describes.

Display changes are applied once the configuration settles. The daemon
loop does not rebuild on each `DisplayChanged` event: its scheduler
(`mosaico_core::daemon`, `daemon/settle.rs`) records it and rebuilds once
no further change has arrived for 750 ms. A KVM switch or a dock sends several
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
3. If a neighbor is found: returns `SpatialTarget::Neighbor(window)`
4. If no neighbor exists: calls `find_adjacent_monitor(direction)` to find
   the nearest monitor in the requested direction by `center_x` comparison.
   Returns `SpatialTarget::AdjacentMonitor(idx)` or `None` if at the edge.

`find_adjacent_monitor(direction)` scans all monitors and picks the one
closest in the given horizontal direction based on work area center_x values.
There is no wrapping -- navigation stops at the leftmost/rightmost monitor.

`find_same_monitor_neighbor(direction)` (for Up/Down) in `navigation.rs`:

1. Computes BSP layout positions
2. Calls `spatial::find_neighbor()` for vertical lookup
3. Returns `Some(window)` or `None` at boundary

## Focus Actions

`focus_direction(dir)`:

- **Left/Right**: resolves horizontal target. If `Neighbor(window)`, focuses
  that window. If `AdjacentMonitor(idx)`, uses `find_entry()` to pick the
  best entry window on the target monitor.
- **Up/Down**: finds same-monitor neighbor. If found, focuses it. Otherwise
  does nothing (stops at boundary).

## Move Actions

`move_direction(dir)`:

- **Left/Right**: resolves horizontal target. If `Neighbor(window)`, swaps the
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

A tiled window the user drags onto another monitor with `[mouse]
drag_to_swap` on joins that monitor's workspace before the tile under the
pointer, or at the end, and that monitor takes focus with it. The monitor is
the one the pointer is on as the drag ends, never the one the window's rect
is mostly on, which the system rescales on its way between monitors of
different scales. One the system maximizes there as it is dropped joins it
too, and stays maximized until it is restored (see
[tiling-layout.md](tiling-layout.md#dragging-windows)). While the window is
dragged over that monitor, the tiles there make room for it at once, as
they would once it joins, and go back once the drag leaves the monitor, for
Escape or a key too; a workspace there in monocle is laid out tiled
meanwhile, and leaves monocle as the window joins it. The window's own
monitor keeps its slot until the drop (see
[tiling-layout.md](tiling-layout.md#tiles-that-follow-a-drag)).

`reassign_monitor(hwnd)` handles the other manual window drags between
monitors: those with `drag_to_swap` off, of windows the tiler did not follow
the drag of, that an action or a display change interrupted, or whose end has
no pointer position:

1. Called when a `Moved` event is received
2. Leaves alone a window on a workspace out of view, as one whose workspace
   the user switched away from while they dragged it: it stays there
3. Queries the window's current monitor via `MonitorFromWindow`
4. If the monitor differs from the workspace's monitor, moves the window
   between workspaces and re-tiles both monitors

## Daemon Initialization

At startup, `daemon_loop()`:

1. Lists the monitors, left to right, with `Enumerator::monitors()`
2. Creates a `MonitorState` for each monitor with an empty `Workspace`
3. Lists the existing windows with `Enumerator::windows()` and assigns them
   to the correct monitor's workspace
4. Applies the initial layout on all monitors

## Design Decisions

- Monitors sorted left-to-right provides intuitive H/L navigation matching
  physical monitor placement. The sort lives in core (`Enumerator::monitors`)
  so every platform numbers monitors the same way.
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

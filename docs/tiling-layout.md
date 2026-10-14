# Tiling Layout

Mosaico ships three tiling layout algorithms -- BSP, VerticalStack, and
ThreeColumn -- that can be assigned per workspace and cycled at runtime. The
layout system is split into platform-agnostic types in `mosaico-core` and
orchestration logic in `mosaico-windows`.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/layout/mod.rs` | `Layout` trait, `LayoutKind` enum, re-exports |
| `crates/mosaico-core/src/layout/bsp.rs` | `BspLayout` -- recursive binary space partitioning |
| `crates/mosaico-core/src/layout/vertical_stack.rs` | `VerticalStackLayout` -- master/stack layout |
| `crates/mosaico-core/src/layout/three_column.rs` | `ThreeColumnLayout` -- center master with side stacks |
| `crates/mosaico-core/src/layout/tests.rs` | Unit tests for all three layouts and `LayoutKind` |
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered collection of window handles per monitor, tracks `LayoutKind` |
| `crates/mosaico-core/src/workspace/monocle.rs` | Monocle state of a `Workspace`, with the windows a display change brought onto it that wait behind the monocle window |
| `crates/mosaico-core/src/rect.rs` | `Rect` -- rectangle type with spatial helpers |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-windows/src/tiling/mod.rs` | `TilingManager`, `MonitorState` -- core orchestration, event/action dispatch |
| `crates/mosaico-windows/src/tiling/navigation.rs` | `focus_direction()`, `move_direction()` -- spatial navigation and cross-monitor moves |
| `crates/mosaico-windows/src/tiling/layout.rs` | `apply_layout_on()`, `retile_all()`, `toggle_monocle()`, `cycle_layout()` -- layout application |
| `crates/mosaico-windows/src/tiling/workspace.rs` | `goto_workspace()`, `send_to_workspace()` -- workspace switching |

### Key Types

- `Layout` (trait) -- defines `fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>`
- `LayoutKind` (enum) -- `Bsp`, `VerticalStack`, `ThreeColumn`; provides
  `next()` for cycling and `name()` for status-bar display
- `BspLayout` -- implements `Layout` with configurable `gap` and `ratio`
- `VerticalStackLayout` -- implements `Layout`: master pane left, vertical
  stack right
- `ThreeColumnLayout` -- implements `Layout`: master pane center, stacks on
  both sides; extra windows alternate left/right
- `Workspace` -- maintains an ordered `Vec<usize>` of window handles for a
  monitor plus a `layout_kind: LayoutKind` field; provides `add()`,
  `insert()`, `remove()`, `swap()`, `compute_layout()`,
  `layout_kind()` / `set_layout_kind()`
- `Rect` -- fields: `x`, `y`, `width`, `height`; methods: `center_x()`,
  `center_y()`, `vertical_overlap()`, `horizontal_overlap()`
- `TilingManager` -- central orchestrator holding `Vec<MonitorState>`,
  `layout_gap`, `layout_ratio`, rules, border, focus state,
  `applying_layout` flag
- `MonitorState` -- per-monitor state: `id`, `work_area`,
  `workspaces: Vec<Workspace>` (up to 8), `active_workspace: usize`

## Layout Algorithms

All three layouts share the same `gap` and `ratio` configuration and the same
gap-handling rules:

- **Outer gap**: applied around the entire work area
- **Inner gap**: half-gap spacing between adjacent windows
- Dimensions are clamped to `max(1)` to prevent zero/negative sizes when gaps
  exceed available space

### Configuration

- `gap: i32` -- pixel gap between windows (default: 8, range: 0-200)
- `ratio: f64` -- split ratio for the primary partition (default: 0.5,
  range: 0.1-0.9)

### BSP (Binary Space Partitioning)

`BspLayout::split()` recursively divides the available area:

1. **One window** -- fills the entire area (minus gaps)
2. **Two or more windows** -- splits the area in half:
   - Even depth: **horizontal split** (left/right)
   - Odd depth: **vertical split** (top/bottom)
3. The first window always gets the larger partition (controlled by `ratio`,
   default 0.5)
4. Remaining windows are recursively laid out in the second partition

```text
+-----------+-----------+
|           |     2     |
|     1     +-----------+
|           |     3     |
+-----------+-----------+
```

### VerticalStack

A master/stack layout with one master pane on the left and remaining windows
stacked vertically on the right:

1. **One window** -- fills the entire area (minus gaps)
2. **Two or more windows** -- the first window gets the left `ratio` portion
   (the master pane); all remaining windows share the right portion, divided
   into equal-height slots

```text
+-----------+-----------+
|           |     2     |
|     1     +-----------+
| (master)  |     3     |
+-----------+-----------+
```

### ThreeColumn

A center-master layout with stacks on both sides. Extra windows alternate
between the left and right stacks:

1. **One window** -- fills the entire area
2. **Two windows** -- master left, second right (behaves like VerticalStack)
3. **Three or more windows** -- the first window occupies the center column
   (width set by `ratio`); subsequent windows alternate into the left and
   right side columns, each divided into equal-height slots

```text
+------+-----------+------+
|  2   |           |  3   |
+------+     1     +------+
|  4   | (master)  |  5   |
+------+-----------+------+
```

## LayoutKind and Cycling

Each `Workspace` stores a `layout_kind: LayoutKind` field that determines
which algorithm is used when tiling its windows. The default is `Bsp`.

### Cycling

The `CycleLayout` action (default binding: `Alt+N`) advances the active
workspace's layout to the next variant:

    Bsp -> VerticalStack -> ThreeColumn -> Bsp -> ...

`LayoutKind::next()` implements this rotation. After cycling, the workspace
is immediately retiled with the new algorithm.

The current layout name is available via `LayoutKind::name()` for status-bar
display (`"BSP"`, `"VStack"`, `"3Col"`).

### Per-Workspace Configuration

The `[layout]` section sets defaults that apply to every workspace, and the
`[workspaces.layouts]` section overrides those defaults for specific workspace
numbers:

```toml
[layout]
gap = 8
ratio = 0.5
default = "bsp"            # default layout for all workspaces

[workspaces.layouts]
3 = "vertical-stack"       # workspace 3 uses VerticalStack
5 = "three-column"         # workspace 5 uses ThreeColumn
```

- `layout.default` -- the `LayoutKind` used for workspaces without an
  explicit override (default: `"bsp"`)
- `workspaces.layouts` -- a map of workspace number (1-8) to `LayoutKind`;
  workspaces listed here start with the specified layout instead of
  `layout.default`

Both fields accept any of the three `LayoutKind` values, serialized in
kebab-case:

| TOML value | Layout |
|------------|--------|
| `"bsp"` | `LayoutKind::Bsp` -- recursive binary space partitioning (default) |
| `"vertical-stack"` | `LayoutKind::VerticalStack` -- master pane left, vertical stack right |
| `"three-column"` | `LayoutKind::ThreeColumn` -- master pane center, stacks on both sides |

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
- `CycleLayout` -> `cycle_layout()` (in `layout.rs`) -- advances the active
  workspace's layout to the next variant and retiles
- `CloseFocused` -> `close_focused()` -- sends `WM_CLOSE`
- `MinimizeFocused` -> `minimize_focused()` -- minimizes the focused window
- `GoToWorkspace(n)` -> `goto_workspace(n)` (in `workspace.rs`) -- switches
  to workspace N on the focused monitor
- `SendToWorkspace(n)` -> `send_to_workspace(n)` (in `workspace.rs`) -- sends
  focused window to workspace N

### Config Reload

The `TilingManager` supports hot-reloading of configuration:

- `reload_config(config)` -- updates the layout gap/ratio, the hiding
  strategy and `BorderConfig`, then retiles every monitor and updates the
  borders. A reload that turns global mode on switches every other monitor
  to the focused monitor's workspace with `goto_workspace_on()`, once the
  new settings are in place; a monitor that switches is laid out as it
  switches, and is not retiled again.
- `reload_rules(rules)` -- replaces the window rules and judges every managed
  window against them again, letting go of the ones they no longer manage. A
  window hidden on another workspace is judged by the rules alone and shown
  again as it is let go (see [configuration.md](configuration.md#hot-reload))
- `adjust_work_areas_for_bar(height, indices)` -- shrinks work areas on
  monitors that display a bar (offsets `y` and `height` by the bar height)
- `reset_and_adjust_work_areas(height, indices)` -- resets to original work
  areas and re-applies bar adjustment (used on bar config reload)
- `bar_states()` -- returns per-monitor `BarState` from `Tiler::bar_states()`:
  workspace count, active workspace, layout, monocle and the focused window
- `restore_all_windows()` -- shows all windows across every workspace and
  monitor; called on shutdown so windows hidden by workspace switching are
  not left invisible

### Layout Application

`apply_layout_on(monitor_idx)`:

1. Prunes stale handles: windows that closed, hid or minimized themselves
   without a report. A window the tiler hid with `hide` is kept while it still
   exists and is still hidden, since a display change bringing it into view
   shows it right after the retile.
2. In **monocle mode**: positions the monocle window to fill the work area,
   and a window a display change brought there behind it, at the same rect
   (see [Monocle Mode](#monocle-mode))
3. In **normal mode**: reads the workspace's `layout_kind` and instantiates
   the corresponding layout (`BspLayout`, `VerticalStackLayout`, or
   `ThreeColumnLayout`) via `compute_positions()`, then emits a placement for
   each window. When the placements are applied, `tiler::apply_window_effect`
   skips windows that are already in place (unless they are maximized), to
   avoid unnecessary repaints, and places and repaints the rest through the
   platform's `WindowControl`.

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
- Tracked by the `monocle: bool` field on `Workspace`

When monocle is active, `apply_layout_on()` skips the normal layout computation
and directly positions the monocle window to fill the monitor's work area with
gap padding. The monocle window is remembered per workspace via the
`monocle_window` field so it persists across workspace switches.

The monocle window stays in front because it is the window that gets focus.
It has focus when monocle is turned on, and a focus report for another window
of the workspace, as clicking it gives, ends monocle (`on_focused`). Focus the
tiler gives itself through `focus_window` does not end it: a window sent to a
monocle workspace, focus following the mouse, or a dialog owned by a window
of the workspace leave monocle on.

A window a display change moves onto a monocle workspace from another
monitor, in view or not, waits behind the monocle window, whether it held
focus or not. The next time the workspace is laid out in monocle, it is put
directly behind the monocle window in the stacking order
(`WindowControl::put_behind`), then placed at the same rect, once. A window
that comes home with a returning monitor does the same on its own workspace.
One that held focus hands it to the monocle window (`settle_focus`), so focus
is on the window in front. A monocle workspace with no window takes the
newcomer as its monocle window, as adoption does, and if that window leaves in
a display change while others wait behind it, the one in front of them becomes
the monocle window. So a window that arrives never turns monocle off, and a
visit from another monitor, with focus or not, leaves the monocle window,
monocle and the stacking as they were.

The workspace keeps the windows that wait (`Workspace::wait_behind`), so a
window that leaves it, by any way, leaves nothing waiting, and leaving monocle
forgets them all.

A window that is always on top, pinned by the user for example, cannot wait
behind a monocle window that is not: `put_behind` keeps each window in its
layer and leaves it in front. It shows at the monocle rect over the monocle
window, which keeps focus and the border, and clicking it ends monocle as
clicking any other window of the workspace does.

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

`goto_workspace(n)` and `send_to_workspace(n)` (in
`crates/mosaico-core/src/tiler/workspace.rs`) are described step by step in
[workspaces.md](workspaces.md#switching-workspaces). The events that hiding
and showing windows cause are kept from undoing a switch as described in
"Hidden Window Tracking" there.

## Design Decisions

- BSP alternates horizontal/vertical splits by recursion depth, producing a
  balanced grid-like layout.
- The first window always gets the primary partition, making the "main" window
  consistently positioned across all three layouts.
- `LayoutKind` is stored per workspace (not per monitor or globally), so
  different workspaces on the same monitor can use different layouts.
- Layout cycling wraps around: `Bsp -> VerticalStack -> ThreeColumn -> Bsp`.
- All three layouts share the same `gap` and `ratio` parameters; only the
  spatial algorithm differs.
- `applying_layout` flag prevents infinite loops from programmatic window
  moves triggering `Moved` events.
- Stale handle pruning runs before every layout application to handle UWP apps
  that close without firing destroy events, and windows minimized without a
  report. It leaves alone a window the tiler itself hid with `hide` and is
  about to show, which is out of view by the tiler's doing, not stale.
- `Workspace` stores a `layout_kind` alongside its `Vec<usize>` handle list
  -- the layout algorithm determines spatial positioning, not insertion order.
- The core layout module is split into sub-files (`mod.rs`, `bsp.rs`,
  `vertical_stack.rs`, `three_column.rs`, `tests.rs`) with one layout per
  file. The tiling orchestration module is similarly split into four files
  (`mod.rs`, `navigation.rs`, `layout.rs`, `workspace.rs`).
- `restore_all_windows()` iterates all workspaces on all monitors to ensure
  no window remains invisible after daemon shutdown.

## Tests

- 5 unit tests for `BspLayout`: single window, two windows, three windows,
  empty input, large gap safety
- 5 unit tests for `VerticalStackLayout`: single window, two windows, three
  windows, five windows, empty input
- 5 unit tests for `ThreeColumnLayout`: single window, two windows, three
  windows (center master), five windows (alternating sides), empty input
- 2 unit tests for `LayoutKind`: cycle order, display names
- 3 unit tests for `Workspace`: add/remove, insert at position, layout
  delegation
- 13 unit tests in `tiling/mod.rs`: event handling, action dispatch,
  workspace switching, config reload, bar state, window restore
- Extensive spatial navigation tests in `spatial.rs` (see
  [monitor-management.md](monitor-management.md))

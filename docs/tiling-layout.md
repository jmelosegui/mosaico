# Tiling Layout

Mosaico ships three tiling layout algorithms -- BSP, VerticalStack, and
ThreeColumn -- that can be assigned per workspace and cycled at runtime. The
layouts, the tiler that applies them and the tiling manager that drives the
tiler are all in `mosaico-core`. `mosaico-windows` implements the platform
contracts the manager runs on.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/layout/mod.rs` | `Layout` trait, `LayoutKind` enum, re-exports |
| `crates/mosaico-core/src/layout/bsp.rs` | `BspLayout` -- recursive binary space partitioning |
| `crates/mosaico-core/src/layout/vertical_stack.rs` | `VerticalStackLayout` -- master/stack layout |
| `crates/mosaico-core/src/layout/three_column.rs` | `ThreeColumnLayout` -- center master with side stacks |
| `crates/mosaico-core/src/layout/tests.rs` | Unit tests for all three layouts and `LayoutKind` |
| `crates/mosaico-core/src/layout/split_ratios.rs` | `SplitRatios` -- the ratio of each split of a workspace's layouts that the user moved, one set per layout, and the layouts laid out with them |
| `crates/mosaico-core/src/layout/split_edges.rs`, `split_resize.rs` | Which split each edge of a tile borders; `resized_splits()` -- where the edges of a tile the user resized put those splits |
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered collection of window handles per monitor, tracks `LayoutKind` |
| `crates/mosaico-core/src/workspace/monocle.rs` | Monocle state of a `Workspace`, with the windows a display change brought onto it that wait behind the monocle window |
| `crates/mosaico-core/src/workspace/splits.rs` | `split_ratios()`, `set_split_ratio()` -- the splits the user moved on a `Workspace`, kept in step with its windows |
| `crates/mosaico-core/src/rect.rs` | `Rect` -- rectangle type with spatial helpers |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-core/src/tiler/mod.rs` | `Tiler` -- every tiling decision, returned as effects; `MonitorState` in `state.rs`; `Tiler::step` in `step.rs` |
| `crates/mosaico-core/src/tiler/placement.rs` | `Timing` and `Transition` -- when a placement reaches the screen, and whether it may animate |
| `crates/mosaico-core/src/tiler/plan_transitions.rs` | `Plan::place()` and the step decision that marks the placements that may animate |
| `crates/mosaico-core/src/tiler/navigation.rs` | `focus_direction()`, `move_direction()` -- spatial navigation and cross-monitor moves |
| `crates/mosaico-core/src/tiler/layout.rs`, `layout_actions.rs` | `apply_layout_on()`, `retile_all()`, `toggle_monocle()`, `cycle_layout()` -- layout application |
| `crates/mosaico-core/src/tiler/workspace.rs` | `goto_workspace()`, `send_to_workspace()` -- workspace switching |
| `crates/mosaico-core/src/tiler/drag.rs`, `drag_mouse.rs` | The drag the tiler follows, keyed on the window the user drags; `Mouse`, what the mouse does |
| `crates/mosaico-core/src/tiler/drag_border.rs`, `drag_end.rs` | The dragged window's border, hidden once it leaves where it started; what interrupts a drag, and what ends one before its `Moved` |
| `crates/mosaico-core/src/tiler/drag_target.rs` | `target()`, `cancelled()` and `ending()` -- where a dropped window lands, and how its drag ended, as plain functions |
| `crates/mosaico-core/src/tiler/drag_drop.rs`, `drag_slots.rs`, `drag_preview.rs` | `on_moved()` -- what a drop does; the slots a drop is decided against; `preview_drop()` -- the preview of where the window lands |
| `crates/mosaico-core/src/tiler/drag_resize.rs` | `move_splits()` -- the splits a tiled window resized by its edges moves |
| `crates/mosaico-core/src/tiler/applier_drag.rs` | The pointer while the user drags a window: left alone, and moved where it was last asked once the drag ends |
| `crates/mosaico-core/src/tiler/manager.rs`, `manager_inputs.rs` | `TilingManager` -- runs the tiler on a `Platform`, event/action dispatch |
| `crates/mosaico-core/src/tiler/manager_frames.rs` | `wants_frames()`, `frame_due(after)`, `frame()`, `turn_off_transitions()` -- what the daemon drives transitions with |
| `crates/mosaico-core/src/tiler/applier.rs` | Applies effects, holds focus changes and moves back, drops their echoes |
| `crates/mosaico-core/src/tiler/applier_run.rs` | Carries out each effect the applier lets through, the ones that reach a moving window included |
| `crates/mosaico-core/src/tiler/applier_motion.rs` | Placements that animate: the motions they become |
| `crates/mosaico-core/src/tiler/applier_frames.rs` | The frame step: posting frames, acting on what the platform found, and placing the windows whose motions are over |
| `crates/mosaico-core/src/tiler/applier_ends.rs` | What ends a motion, effect by effect and event by event, and where the pointer goes |
| `crates/mosaico-core/src/tiler/applier_stop.rs` | Ending motions at once: every one as the tiler stops or transitions turn off, and those whose frames failed; and, while transitions are off, having the platform forget the windows it gave frames to |
| `crates/mosaico-core/src/tiler/applier_display.rs` | What a display change does to moving windows: each stops where it is and keeps its target, where it is placed once the change is rebuilt for |
| `crates/mosaico-core/src/tiler/applier_borders.rs` | The borders of moving windows |
| `crates/mosaico-core/src/tiler/applier_posted.rs`, `applier_reports.rs` | What may still be on its way to a window, which the applier does not read back as done, and what a report of the window says of it, its echoes included (`Applier::echo`) |
| `crates/mosaico-core/src/tiler/applier_landing.rs` | The windows a posted placement is taking somewhere, until they land: on a report, or at `LANDING_WAIT` |
| `crates/mosaico-core/src/tiler/applier_shows.rs`, `applier_view.rs` | The changes of show state on their way to windows, and the tiler's view of the desktop with each window as the change leaves it |
| `crates/mosaico-core/src/tiler/plan_maximized.rs` | Whether a window placed in a step is still maximized after it, with its changes posted |
| `crates/mosaico-core/src/tiler/applier_transitions.rs` | What the applier keeps for transitions, and the lines it logs of them |
| `crates/mosaico-core/src/motion/policy.rs` | `step_animates()`, `still_animates()` -- which placements animate; `step_snap()`, `apply_snap()` in `policy_snap.rs` -- why the others snap |
| `crates/mosaico-core/src/motion/motions.rs` | `Motions` -- the table of moving windows; each frame and what it found in `motions_frame.rs` |
| `crates/mosaico-core/src/motion/journal.rs` | `Journal` -- what the log is told of each motion |
| `crates/mosaico-core/src/tiler/platform.rs` | `Platform` -- what the tiling manager needs from the OS |
| `crates/mosaico-windows/src/tiling/platform.rs` | `Win32Platform` -- the Win32 implementation of each of those contracts |

### Key Types

- `Layout` (trait) -- defines `fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)>`
- `LayoutKind` (enum) -- `Bsp`, `VerticalStack`, `ThreeColumn`; provides
  `next()` for cycling and `name()` for status-bar display
- `BspLayout` -- implements `Layout` with configurable `gap` and `ratio`;
  `apply_splits()` gives each split a ratio of its own
- `VerticalStackLayout` -- implements `Layout`: master pane left, vertical
  stack right
- `ThreeColumnLayout` -- implements `Layout`: master pane center, stacks on
  both sides; extra windows alternate left/right
- `SplitRatios` -- the ratio of each split the user moved, one set per
  layout: `ratio()`, `set()`, and `apply()`, which lays windows out with them
  (see [Split Ratios](#split-ratios))
- `Workspace` -- maintains an ordered `Vec<WindowId>` of the windows on a
  monitor plus a `layout_kind: LayoutKind` field and its `SplitRatios`;
  provides `add()`, `insert()`, `remove()`, `swap()`, `compute_layout()`,
  `layout_kind()` / `set_layout_kind()`, `split_ratios()` /
  `set_split_ratio()`
- `Rect` -- fields: `x`, `y`, `width`, `height`; methods: `center_x()`,
  `center_y()`, `vertical_overlap()`, `horizontal_overlap()`
- `TilingManager<P: Platform>` -- runs a `Tiler` on the platform `P`: feeds
  it actions and events, applies the effects it returns, holds focus changes
  and moves back while input keeps arriving, and drops the events its own
  effects cause
- `Platform` -- one implementation of each contract the manager uses
  (`Desktop`, `WindowControl`, `Decorations`, `Enumerator`) and the time
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
  range: 0.1-0.9), where every split is until the user moves it

### Split Ratios

Split `k` divides windows `k..` into window `k` and the rest. Each split of
the BSP chain is one, and in VerticalStack and ThreeColumn split 0, at the
master's edges, is the only one. Its ratio is the share of its area that
its first window, the master in those two, gets.

Each `Workspace` keeps a `SplitRatios` (`layout/split_ratios.rs`), one set
per layout, with the ratio of each split the user moved, by dragging an edge
(see [Resizing Splits](#resizing-splits)). Every other split is at `ratio`.
So cycling layouts finds each layout's own splits again, monocle and
retiling leave them alone, and a config reload with a new `ratio` moves the
splits nobody moved and leaves the others where the user put them. A split
is kept within 0.1 and 0.9 (`MIN_SPLIT_RATIO`, `MAX_SPLIT_RATIO`), as
nothing tells the tiler how small a window can be.

The workspace keeps the BSP chain in step with its windows
(`workspace/splits.rs`):

- a window added or inserted at `k` gets a split at `ratio` at split `k`, or
  at the end of the chain for the last window, and the splits after it go
  along with their windows;
- a window removed takes its own split with it, split `k`, or the last one
  for the last window, so the other windows keep theirs;
- a swap changes no split: the slots keep their sizes, and the windows
  trade places.

A split that moves along the chain, as a window before it comes or goes,
keeps its ratio but takes the direction of its new place: side by side at an
even `k`, one above the other at an odd one. So of three windows, a second
one given a quarter of the right half's height has a quarter of the width
once the first one closes. A monitor that disconnects and comes back gives
its windows back the splits of the chain they had there, once they are all
back in their places, whichever layout is on screen (`reconnect.rs`,
`reconnect_restore.rs`).

The master's ratio in VerticalStack and ThreeColumn is not tied to a window,
and stays as windows come and go, and while its monitor is away. The
ThreeColumn master stays centered: its ratio is the share of the width it
gets, and both its edges move when it changes. A window dropped on another
monitor gets a split of its own there, at `ratio`, and the slot the preview
shows for it counts that split.

### BSP (Binary Space Partitioning)

`BspLayout::split()` recursively divides the available area:

1. **One window** -- fills the entire area (minus gaps)
2. **Two or more windows** -- splits the area in half:
   - Even depth: **horizontal split** (left/right)
   - Odd depth: **vertical split** (top/bottom)
3. The first window always gets the larger partition (controlled by the
   split's ratio, `ratio` until the user moves it; `apply_splits()` takes
   the ratio of each)
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
   (the master pane), or the share the user moved its edge to; all remaining
   windows share the right portion, divided into equal-height slots

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
   (width set by `ratio`, or by where the user moved one of its edges);
   subsequent windows alternate into the left and right side columns, each
   divided into equal-height slots

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
is immediately retiled with the new algorithm. A left click on the layout
widget of a monitor's bar cycles that monitor's layout, and a right click
toggles its monocle (see [status-bar.md](status-bar.md#widget-clicks)).

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

The `TilingManager` is in `mosaico-core`, generic over the `Platform` it
runs on, so its tests run on a fake platform on any OS. The Windows daemon
builds it on `Win32Platform` in its daemon loop. It processes both window
events and user actions.

### Event Handling

`handle_event()` dispatches `WindowEvent` variants:

- `Created` -- moves a window that is not shown yet into the slot it will
  get; adds a visible one to the focused monitor's workspace if tileable,
  moving it ahead of the layout (see
  [Where a new window first appears](#where-a-new-window-first-appears))
- `Destroyed` -- removes window from whichever workspace contains it
- `Focused` -- updates focus tracking, shows/hides border
- `MoveStarted` -- starts following the drag of a tiled window (see
  [Dragging Windows](#dragging-windows))
- `Moved` -- drops the window the tiler followed the drag of where the
  pointer is, or detects manual monitor reassignment (drag to different
  monitor)
- `Minimized` -- removes window from workspace, re-tiles, unless the window
  is no longer minimized, as when the report comes after it was restored
- `Restored` -- re-adds window to workspace, moving it ahead of the layout
  like a new one

A window that leaves the layout gets none of the moves or focus changes
still held back for it from a burst of key presses, whether it was
destroyed, minimized or hidden, new rules stopped managing it, or the
layout found it gone or minimized. `untile()` (in `removal.rs`), which
every one of those goes through, emits `Untile` for it, and the tiling
manager drops them, whether or not the window has a border to drop. A
placement made at once for a window still lets a focus change held for
it land.

### Action Handling

`Tiler::handle_action()` (in `actions.rs`) dispatches `Action` variants to
dedicated methods in the split tiler modules. An action from a bar first
moves focus to the bar's monitor when it acts on a monitor, and its step
moves no pointer (see [actions.md](actions.md#origins)):

- `Focus(dir)` -> `focus_direction(dir)` (in `navigation.rs`) -- spatial
  navigation. Left/Right use `resolve_horizontal_target()` and can cross
  monitors. Up/Down use `find_same_monitor_neighbor()` and stay on the
  current monitor.
- `Move(dir)` -> `move_direction(dir)` (in `navigation.rs`) -- spatial swap.
  Left/Right can move windows across monitors. Up/Down swap on the same
  monitor.
- `Retile` -> `retile_all()` (in `layout_actions.rs`) -- re-applies layout
  on all monitors
- `ToggleMonocle` -> `toggle_monocle()` (in `layout_actions.rs`) -- toggles
  monocle mode
- `CycleLayout` -> `cycle_layout()` (in `layout_actions.rs`) -- advances the
  active workspace's layout to the next variant and retiles
- `CloseFocused` -> asks the platform to close the focused window, which on
  Windows posts `WM_CLOSE`
- `MinimizeFocused` -> asks the platform to minimize the focused window
- `GoToWorkspace(n)` -> `goto_workspace(n)` (in `workspace.rs`) -- switches
  to workspace N on the focused monitor
- `GoToWorkspaceNext` / `GoToWorkspacePrev` -> `step_workspace(1)` /
  `step_workspace(-1)` (in `actions.rs`) -- switches to the workspace after or
  before the one on screen, following `[workspaces] wrap`
- `SendToWorkspace(n)` -> `send_to_workspace(n)` (in `workspace.rs`) -- sends
  focused window to workspace N

### Config Reload

The `TilingManager` supports hot-reloading of configuration:

- `reload_config(config)` -- updates the layout gap/ratio, the hiding
  strategy, `BorderConfig`, whether the mouse follows focus, whether a drop
  moves a dragged window and whether a resize moves splits, and the
  `[animations]` settings, which the applier gets too, then retiles every
  monitor and updates the borders. A new ratio moves the splits the user
  did not move, and leaves the others. A reload that turns global mode on
  switches every other monitor to the focused monitor's workspace with
  `goto_workspace_on()`, once the new settings are in place; a monitor that
  switches is laid out as it switches, and is not retiled again.
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
  monitor, once the windows still moving have been placed at their targets
  (see [Placements That Animate](#placements-that-animate)); called on
  shutdown so windows hidden by workspace switching are not left invisible

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

When a window that is already on screen is taken in (`add_and_focus`,
`try_adopt`, or a window dropped onto a monitor), `move_before_layout` first
posts it a move to its slot with `PrePlace`, which does not wait for its
application. The layout pass then runs as usual, placing the windows in the
layout's order. The new window's own placement there is made in full even
when the posted move has already landed, so the window still gets the
placement and the repaint the layout pass would have given it. A window
pre-placed while it was hidden keeps the usual check, and is left alone if
it is in place.

A layout never sets off another one: `WindowEvent::Moved` is only reported
when the user finishes dragging a window, never for the tiler's own
placements, and the `LocationChanged` reports they cause are only acted on
when the focused window is maximized or restored.

### Placements That Animate

Every `Effect::Place` carries a `transition`: `Snap`, the window is put in
place at once, or `Animate`, it may move there over a short time. The layout
passes place windows through `Plan::place`, which records the monitor each
placement lays its window out on, and every placement starts as `Snap`. As
the step finishes, `Plan::finish` (`plan.rs`) runs the step decision,
`motion::step_animates`, over the placements of the step (`mark` in
`plan_transitions.rs`) and marks the ones it allows `Animate`. It needs facts
only the step has:

- whether transitions are on (`[animations]` enabled, with a duration above
  zero), which the tiler takes from `Settings` at start and from every
  config reload, and which stay off in a session without frames, whatever
  the config says (see below);
- whether the step shows the window, moves it ahead of the layout
  (`PrePlace`) or takes it in (`Plan::take_in`, from `add_and_focus`,
  `try_adopt` and a window dropped onto a monitor): such a window appears
  where it goes instead of sliding there;
- whether the window is put behind another before its placement, as the
  windows waiting behind a monocle window are;
- whether the step is a display change, the tiler's start, or the bar
  reservation right after it (`Tiler::new` for the start, `Tiler::step` in
  `step.rs` for the others): everything snaps while monitors and work areas
  are in flux;
- whether the monitor the placement lays the window out on is the one the
  desktop has it on: a window that changes monitors snaps;
- how many windows the step's placements move from where the desktop has
  them: a step that moves more than 16 snaps them all.

With transitions off nothing is marked and no desktop is asked anything, so
the effects are the same as before apart from the field. The coalescer keeps
the mark of each placement it holds and releases it with it.

The applier checks a marked placement again as it applies it
(`applier_motion.rs`), against what the desktop says then, with
`motion::still_animates`: the system allows animations (while `follow_system`
is on), the window is neither minimized nor maximized, some edge of it travels
more than 4 pixels, and the platform can move windows through frames. No
placement animates while a display change settles (see below). A
placement that passes becomes a motion in the applier's `Motions` table. A
window at rest starts from where it was last sent while that may still be on
its way, as to a busy window, and otherwise from where the desktop has it,
along the easing, with the duration configured then. A target the window
already has changes nothing while its placement still animates, and a new
one mid-flight continues from where the motion has the window, keeping as
much of its speed as does not carry it past the target, over the duration
configured then. The applier takes the `[animations]` settings at start and
from every reload. A motion in flight keeps the duration and easing it
started with while its target stays; one that a reload's retile sends
elsewhere turns towards its new target over the new duration. A reload that
turns transitions off snaps every moving window its retile places, whether
its target changed or not. Any other placement of a moving window, one that
no longer animates included, ends its motion and places it at once, in full.

The windows an `apply` or a `flush` sets moving form a step
(`Motions::begin_step`).

The frame step (`applier_frames.rs`, `TilingManager::frame()`) posts every
frame due in one `place_frames` call, and acts on what the platform found of
each window (`motion/motions_frame.rs`):

- A window that handled the frame before moves on, and its border is moved
  to where the frame found it, or, when it took the frame posted at the step
  before exactly, to the frame now posted: the border neither trails a
  window that keeps up nor runs ahead of a slow one. A window that changes
  size on its way has its border hidden until it is in place instead (see
  [focus-border.md](focus-border.md#borders-of-moving-windows)). One still to
  handle its frame stays, and so does its border.
- A window found still to handle its frame and not seen to move for 50 ms
  (`MAX_LAG`), since its motion started or since it handled its first
  frame, has fallen behind: every motion of its step, and no other, ends at
  once with a full placement at its target. The limit is a time, so it is
  the same at any refresh rate. A window that handled its first frame
  counts from then, so a first frame step that came late does not make it
  fall behind; one still to handle its first frame counts from the start of
  its motion.
- A window maximized or minimized meanwhile stops where it is, and one that
  is gone is forgotten. What its motion kept for its border is dropped.
- A motion ends at its end time with a full placement at its exact target,
  made even when the window is there already, and its border is drawn once
  there: at once, as the last frames usually took the window there, or once
  the window lands, as below.

No placement waits for a frame to land: the platform orders a placement,
a repaint, `put_behind` and a change of how a window is shown other than
cloaking, behind what was posted to the window before, so a placement made
while a frame is on its way lands after it; `focus` is never posted. A
window whose application stopped responding after it was given frames
needs nothing of its own: its placement is posted like its frames. One that
had stopped before any was gets nothing posted, and is placed as it is
without transitions: the call is made at once, and waits for it.

While something posted to a window may still be on its way
(`FrameControl::posts_behind`), the desktop may not show it yet, so the
applier does not read it back as done (`applier_posted.rs`). Where it keeps
its own record of how a change was made, it asks right after the change: a
posted change renews the grace, so the answer then is true for a change
that was posted, or for a window whose application stopped responding
since, which waits the same way.

- A placement is not skipped because the desktop has the window in place
  already, unless the window was last sent there too, by a placement or a
  frame: what is still on its way would take it elsewhere. So a swap undone
  before a slow window got to it puts the window back.
- `Focus { move_cursor: true }` and `CenterCursor` aim at the center of
  where the window was last sent, as they aim at a moving window's target,
  and a new motion of the window starts from there.
- A posted placement that finds the window there already, within 4 pixels
  on every edge, and where the last thing sent to it took it, to the pixel,
  moves nothing, so it reports nothing and needs no second repaint: the
  border its motion kept is drawn at once. Any other is waited for
  (`applier_landing.rs`), one whose last frame left the window a pixel
  short too, as the placement moves it that pixel and it reports. The
  window is repainted again once it lands, as a posted repaint only repairs
  what was wrong before the changes queued ahead of it, and the border its
  motion kept is drawn then, around where it was placed; until then the
  border stays where the window was last found. It lands once it reports
  being there, within 4 pixels, as a window that rounds its own size is;
  once it reports being anywhere while nothing sent to it may still be on
  its way; and when `LANDING_WAIT` (500 ms, twice `PENDING_LIMIT`) after
  the placement has passed, if it is there or has moved since the
  placement was posted, as a window that settles further away does, a
  console sized to whole text cells say, or one whose report was missed. A
  window that has not moved by then, as its application has not got to
  the placement, is no longer waited for by time, so the daemon does not
  wake for it: it lands on its next report, which comes once its
  application gets there. A new motion of the window, or one that stops
  on its way, ends the wait, and so does the user grabbing the window. A
  `Border` effect for a window still to land, after any placement posted
  behind frames, one that snaps too, replaces the style drawn then; a hide
  happens at once.
- A posted hide, minimize or show that changes something has the window as
  it leaves it in the tiler's view of the desktop (`applier_shows.rs`), until
  the desktop has the window that way, the window reports the change, or
  nothing sent to the window may still be on its way. A show is kept as
  long as a hide of the window is still expected to be reported, as it is
  still to land. So a display change restores a window whose minimize is
  queued before it places it, and a layout pass does not take a window
  whose show is queued for one the user hid. A show of a window already
  shown changes nothing, and is not kept. A hide or show by cloaking never
  waits, so it is never kept.
- A window the tiler places and that is out of view after the step, while
  its changes are posted, counts as maximized as long as the desktop says
  so (`plan_maximized.rs`): the platform may leave it maximized. A display
  change shows a hidden maximized window whose changes are posted before it
  places it, so the placement lands behind the show and ends the state, as
  it restores a window still minimized first. A cloaked window counts as
  shown, and is placed first as any other: a posted placement ends its
  maximized state as it does a shown window's.

`TilingManager::wants_frames()` says whether any window is moving, and
`frame_due(after)` when the first motion is due to end, or the first wait by
time for a window to land ends, after `after`, the last frame step.
When `place_frames` returns `Unsupported`, the applier logs it once, places
every moving window at its target at once, and lets nothing animate from
then on; `TilingManager::turn_off_transitions()` does the same for a daemon
that cannot have display frames. Either way the tiler then has transitions
off in the step decision (`Tiler::turn_off_transitions`), whatever a reload
says, so every placement is marked to snap. Any other error is logged once
until frames work again, and places the windows the frames were for at
their targets, as placements that snap do.

While transitions are off, by the config, by the system's setting it
follows, or because the platform cannot move windows frame by frame, the
applier asks the platform to forget the windows it gave frames to
(`FrameControl::settle_responding`), so that changes to them are made at
once again: as they turn off, and at the start of every step after, before
the tiler reads the desktop, and of every flush, for as long as one of
those windows is still posted to, as it is within the grace or while its
application does not respond (`applier_stop.rs`). So a step decides as it
would from startup once the windows have settled.

Other effects that reach a moving window (`applier_run.rs`, with the events in
`Applier::echo`, in `applier_reports.rs`, and the endings in
`applier_ends.rs`):

- `Hide`, `Minimize`, and the window's `Destroyed` and `MoveStarted`
  events, which `Applier::echo` sees first, stop the motion where the window
  is, `Hide` and `Minimize` before they are made, as a frame posted after
  them would land after them. So does a `Minimized` or `Hidden` report that
  is not the tiler's own, when the desktop agrees: the user minimized the
  window or its application hid it, and the `Untile` that follows has
  nothing to finish. The tiler ignores a hide report from a window still
  shown, as an application recreating its surface sends, and a minimize
  report from one restored again, and the window keeps moving. A window
  shown or restored later is placed again by the tiler, and that placement
  snaps.
- `Close` leaves it moving: the application may refuse to close, and its
  `Destroyed` or `Hidden` report ends the motion.
- `Untile` and `Reveal` finish the motion at its target with a full
  placement. A window the tiler lets go of gets no border drawn there: the
  step that lets go of it drops its border. The tiler stopping places every
  moving window, and every window a display change stopped, at its target
  before it reveals any, and draws no border there (`applier_stop.rs`).
- A display change stops every motion where its window is as soon as it is
  noted (`TilingManager::display_changing()`), and each window keeps its
  target (`applier_display.rs`). Until the change is rebuilt for, every
  placement snaps, as the layout it is for is about to be replaced. After
  the rebuild, or when the daemon cannot read the monitors and skips it
  (`TilingManager::display_change_skipped()`), each window stopped that
  nothing placed or took out of view meanwhile is placed at its target, and
  the border its motion kept is drawn there.
- `Focus { move_cursor: true }` and `CenterCursor` put the pointer at the
  center of the window's target, not where it is on its way there.
- `MouseHover` is dropped while windows move, and is not repeated until the
  pointer moves to another window. That rarely matters: a report takes three
  polls of the pointer, at least 200 ms, longer than a default 180 ms
  transition, so a window sliding under a pointer at rest is usually
  reported only once it has stopped.

With debug logging, the step decision logs why a step or a placement that
moves its window snaps, from `motion::step_snap` (`plan_transitions_log.rs`),
and the applier logs the windows each step sets moving or turns, why a
placement snaps as it is applied (`motion::apply_snap`), or because a display
change settles, a window that falls behind, as the frame step decided
(`Tick::behind`), how each motion ends and how the placement that ended it
was made, each step once its last motion has ended or a later step has
turned it, with its slowest window, each window a display change stopped as
it is placed at the target it kept, and each window a transition placed
behind what was on its way as it lands, on a report or once the wait for it
by time is over: from the `motion::Journal` it keeps of each motion and its
record of what was posted, never a line per frame (see
[animations.md](animations.md#log-lines)).

### Window Tileability

`Tiler::is_tileable(window)` (in `adopt.rs`) checks:

1. The window is visible
2. The platform offers it as a candidate (`Desktop::candidate`), which rules
   out tool windows, owned dialogs, shell surfaces, windows the system keeps
   out of view and windows the platform cannot position
3. The rules manage it: `should_manage()` for a window with a title bar, and
   only a rule that asks for it (`matching_rule()`) for one without

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
focus or not. The next time the workspace is laid out in monocle, the monocle
window is placed first, then the window is put directly behind it in the
stacking order (`WindowControl::put_behind`) and placed at the same rect,
once. A window that comes home with a returning monitor does the same on its
own workspace. One that held focus hands it to the monocle window
(`settle_focus`), so focus is on the window in front. A window hidden with
`hide` is put behind again once it is shown, in case showing it brought it to
the front. A monocle workspace with no window takes a newcomer as its monocle
window, as adoption does: the one that has focus, or else the first. If that
window leaves in a display change while others wait behind it, the one with
focus or else the one in front of them becomes the monocle window, counting
the windows that come in the same change. So a window that arrives never turns
monocle off, and a visit from another monitor, with focus or not, leaves the
monocle window, monocle and the stacking as they were.

A workspace whose monocle window the user sent or dragged away stays in
monocle for it to come back, but is tiled meanwhile. A window that lands there
is tiled with the others and keeps focus, as on any tiled workspace.

The workspace keeps the windows that wait (`Workspace::wait_behind`), so a
window that leaves it, by any way, leaves nothing waiting, and leaving monocle
forgets them all.

A window that is always on top, pinned by the user for example, cannot wait
behind a monocle window that is not: `put_behind` keeps each window in its
layer and leaves it in front. It shows at the monocle rect over the monocle
window, which keeps focus and the border, and clicking it ends monocle as
clicking any other window of the workspace does.

## Dragging Windows

A drag is about the window the platform reports being dragged, never
whichever window has focus. `WindowEvent::MoveStarted` carries what the
pointer grabbed (`Grab::TitleBar`, or the `Edges` of the frame), the
window's rect, whether it is maximized and the pointer, and `Moved` the
pointer, all read as the platform saw the drag start and end (see
[window-events.md](window-events.md#what-a-drag-carries)).

### While a Window Is Dragged

`MoveStarted` for a window tiled on the workspace on screen starts a drag
the tiler follows (`drag.rs`), keyed on that window. A drag that started on
a maximized window, as the platform saw it then, is a move, whatever the
platform says was grabbed, and the rect it started at comes from the
desktop when the platform could not read it. Until the drag ends:

- the window keeps its slot in the layout, so a window that opens is tiled
  around it, but no layout pass places it, so it never animates either,
  whatever the user does with the keyboard or the bars meanwhile;
- its border is hidden once a position report finds it away from where it
  started, as it would only show where the window was (`drag_border.rs`).
  A click on the title bar, which the platform reports as a drag that
  moves nothing, leaves it drawn;
- its position reports only hide its border and move the preview of where
  it would land, never the maximize logic that would put it back in its
  slot;
- the tiling manager drops what was held back for it from a burst of keys
  as the drag starts, as for a window that leaves the layout, and leaves the
  pointer alone while any window is dragged, tiled or not
  (`applier_drag.rs`): a focus change is made at once, and the pointer goes
  where the last one asked for once the drag has ended and its end has been
  handled, unless focus has moved on from there without the pointer, as a
  click or the wheel on a bar moves it. A dragged window that closes, hides
  or is minimized gives the pointer back, as does another window taking
  focus other than by the tiler's doing: the same focus reports as end a
  drag (`drag_end.rs`), so a focus change the tiler makes without the
  pointer, as for a bar on another monitor, and its report keep the pointer
  held, and a key after them moves it only once the window is let go;
- focus follows mouse rests while any window is dragged, tiled or not: the
  tiling manager drops each report of the pointer resting on a window
  (`applier_reports.rs`), as a focus change then would reach the system's
  move or size loop, and the next report after the drop applies as usual.

Every action that reaches the tiler, as `Input::Action`, from a key, a
command, or a click or the wheel on a bar, whatever monitor it is for, and
a display change from when it is noted, interrupt the drag (`drag_end.rs`):
its preview is hidden and its `Moved` is handled as for a drag the tiler
did not follow, but the window stays the user's until then, and no layout
pass places it. `toggle-pause`, from a key or the **PAUSED** mark, is
handled by the daemon (`daemon_loop_commands.rs`) and never reaches the
tiler, so it leaves the drag alone. A display change that brings the window
onto a monocle workspace leaves it waiting there, with its focus, until it
is dropped (`display_monocle.rs`). A workspace switch of its monitor, as
Alt+2 or the wheel over its bar, hides it with the rest of its workspace,
and its drop leaves it there. The drag ends early, as for one the tiler did
not follow, when the window leaves the workspace on screen of its monitor,
when another window takes focus other than by the tiler's doing, and when
the tiler stops. A dialog of the window the tiler gave focus, which the
system gives focus in its place, is the tiler's doing. A second
`MoveStarted` takes the place of a drag whose end was never reported.

The platform may not report the end of every drag (see
[window-events.md](window-events.md#how-events-drive-tiling)). One whose end
is lost is over at the first of these. Until then, as when the user goes on
with the keyboard alone, its window is not laid out, its border stays
hidden, the pointer does not follow focus, and focus does not follow the
pointer.

### Where a Window Lands

With `[mouse] drag_to_swap`, on by default, `Moved` ends the drag
(`drag_drop.rs`). How it ended is decided first (`drag_target::cancelled`
and `drag_target::ending`):

- a window left within a few pixels of where it started, or back in its
  slot, was cancelled with Escape, and is handled as before, even with the
  pointer over another tile. A few pixels, and the slot, count too, as the
  start may have been read once the window began to move;
- a drag whose pointer ends within a few pixels of where it started was a
  click on the title bar, and is handled as before, even when a double
  click maximized the window by the time its first click ends. A few pixels
  are what a press may wobble before Windows starts a drag, as many as
  `motion::hardly_apart` allows. So a click or Escape also leaves alone a
  window wider than its slot, whose title bar reaches over the next tile;
- a drag by an edge resizes, and moves the splits its edges border, as
  [Resizing Splits](#resizing-splits) says, before any of this is decided;
  one whose grab the platform could not tell resized when the window's size
  changed, and is handled as before: the window goes back to its slot;
- a move that left the window maximized on its own monitor, or changed its
  size by more than a few pixels on an edge while the window lay on its own
  monitor, its center there, was snapped by the system, and the window goes
  back to its slot. A move keeps a window's size but for the system
  rescaling it on a monitor of another scale, so one that changed size with
  most of it on another monitor has moved, whichever tile the pointer is
  over, and one the system maximized there joins that monitor's workspace
  and stays maximized until it is restored, as a window the user maximizes
  does. A snap that leaves
  the window its own size, as to the half of the monitor the other of two
  tiles fills without gaps, is taken for a move onto that tile, which
  leaves the window where the system put it;
- any other drag moved the window. A window that was maximized as the drag
  started has moved too, though it changes size as it leaves maximized.

A window that moved lands where the pointer drops it (`drag_target::target`),
from the pointer, never from the window's rect:

| The pointer is | The window |
|----------------|------------|
| over another tile of its monitor | takes that tile's slot, and the other window its own |
| over its own slot, a gap or the bar, or anywhere on its monitor in monocle | goes back to its slot |
| over a tile of another monitor | joins that monitor's workspace before that tile, and the monitor takes focus with it |
| over a gap of another monitor, its bar, or a monocle workspace there | joins the end of that workspace; on a monocle workspace, only the monocle window is placed and nothing is put behind it, as for any window dragged onto one |
| on no monitor, as over a taskbar | is handled as before |

A monitor counts with the bar's strip (its work area as the platform reports
it), and in global mode the window joins the workspace on screen there, as
every monitor shows the same number. The window keeps focus, and the
pointer stays where the user dropped it. The drop's placements may animate
like any other, as both windows of a swap stay on their monitor, and a
window the transitions had sent somewhere moves from where it was dropped.

A drag the tiler did not follow, as of a window it does not manage, one
that was interrupted, one whose end has no pointer, and every drop with
`drag_to_swap` off are handled as before (`reassign_monitor`): the window
goes back to its slot, joins the end of the workspace of the monitor it is
on now, or is taken in there when the tiler would tile it, and a window the
tiler leaves alone changes nothing under it. A window whose workspace is
out of view, as after Alt+2 mid-drag, stays on that workspace.

### Resizing Splits

With `[mouse] resize_splits`, on by default, `Moved` for a drag by edges of
the frame moves the splits those edges border (`drag_resize.rs`), whatever
`drag_to_swap` says. `layout::resized_splits` finds which split each edge
borders (`split_edges.rs`) and where the edge, as the window is when the
drag ends, puts it (`split_resize.rs`):

| Layout | The edges that move a split |
|--------|-----------------------------|
| BSP | the far edge of window `k`, its right one for even `k` and its bottom one for odd `k`, moves split `k`, unless it is the last window; its near edges move splits `k - 1` and `k - 2`, which it lies beyond |
| VerticalStack | the master's right edge, and the left edge of each window of the stack, move split 0 |
| ThreeColumn | with two windows, as in VerticalStack; with more, the master's two edges, and the inner edge of each window of a side column, move split 0, and the master stays centered, so both its edges follow |

A corner where two splits meet moves both. A held edge dropped within 4
pixels (`motion::MIN_TRAVEL`) of where its slot has it, or of where it was
as the drag started, moves no split, as a press may wobble that much: a
corner dragged along one of its edges leaves the other split as it was,
still at `ratio` when nobody moved it. The new ratio is where the edge is,
as a share of the split's area, with half a pixel more so that the layout,
which rounds down, puts the edge back on the pixel it was dropped at, and
it is kept within 0.1 and 0.9. A window gone or minimized without a report
leaves the layout first (`prune_stale_handles`), so the splits are worked
out against the windows that stay. The monitor is then laid out again, the
resized window included: the windows beyond the split follow the edge, and
the window's edges that border no split, its outer edges and those between
two windows of a stack or of a side column, go back. The placements may
animate like any other, from where the user let go. The ratio comes from
where the window is, so a resize whose end has no pointer, or that ends
with the pointer over a bar, moves its split all the same.

A resize of 4 pixels or less on every edge goes back to the slot, as a
click or a cancel would: `drag_target::cancelled` takes a window left
within 4 pixels of where it started, or a pointer that ends within 4 pixels
of where it pressed, for Escape or a click on an edge. A resize in monocle,
one by edges that border no split, one the tiler did not follow or that an
action, from a key, a command, or a click or the wheel on a bar, or a
display change interrupted, one whose grab the platform could not tell, and
every resize with `resize_splits` off are handled as before too: the window
goes back to its slot. `toggle-pause` never reaches the tiler, so it leaves
a resize alone, as it does any drag. A window left maximized as the drag
ends stays maximized and moves no split, as at the end of any drag. A
resize shows no preview.

Escape puts the window back where the system started the drag, which moves
no split, as that is within 4 pixels of the start the platform read, unless
the platform read the start more than 4 pixels into the resize, while the
window was off its slot, as on its way there or held wider by its minimum
size. Then the window put back counts as resized to where it started, and
its split moves there. The same goes for the edge of a corner the user did
not move: one that ends more than 4 pixels from both the start read and
its slot, as when the start was read late on a window off its slot, moves
its split there.

### The Drop Preview

While a window is dragged with `drag_to_swap` on, each of its position
reports works out where it would land if dropped now, with the pointer
where it is, by the rule the drop follows (`drag_preview.rs`), and the
tiler emits `Effect::ShowPreview` with the slot it would get: its own, that
of the tile under the pointer, or the one it would get on another monitor
once put before the tile under the pointer, which in BSP is not that tile.
So a window a few pixels off its size previews the tile it would swap
with, and one whose size changed more on its own monitor previews its own
slot. The tiler emits nothing while that slot stays the same. A resize, a
drop that would place the window nowhere, as onto another monitor's
monocle workspace, a window back where it started, and a pointer off every
monitor hide it (`Effect::HidePreview`), and so does the end of the drag or
its interruption, however it comes. A focus change that leaves the drag
going shows it again, which puts it back just behind the dragged window.
Without position reports while the window is dragged, as with window
contents hidden while dragging, there is no preview, and the drop decides
from where the window and the pointer are at the end. The preview is drawn
in the focused border's color, as wide and as round as the borders, through
`Decorations::show_preview` (see
[focus-border.md](focus-border.md#drop-preview)).

## Workspace Management

Each monitor has up to 8 workspaces (`MAX_WORKSPACES = 8`), stored as a
`Vec<Workspace>` in `MonitorState`. One workspace per monitor is active at a
time (`active_workspace` index).

The `Workspace` type maintains window ordering:

- `add(window)` -- appends to the end (no duplicates)
- `insert(index, window)` -- inserts at a specific position (index clamped
  to length, no duplicates). Used when moving windows across monitors to
  control BSP slot placement.
- `remove(window)` -- removes and shifts the remaining windows
- `swap(a, b)` -- swaps two windows by index
- `compute_layout(gap, ratio, work_area)` -- lays the windows out with the
  workspace's layout through its `SplitRatios`: each split where the user
  moved it, or at `ratio`
- `windows()` -- returns a slice of all the windows in the workspace
- `split_ratios()` and `set_split_ratio(split, ratio)` -- where the user
  moved the splits; `add`, `insert` and `remove` keep them in step with the
  windows, and `swap` leaves them (see [Split Ratios](#split-ratios))

New windows are always placed on the **focused** monitor's **active
workspace**, not the OS-determined monitor. When moving a window to an
adjacent monitor, `insert(0, window)` is used when entering from the right
(leftmost BSP slot) and `add(window)` when entering from the left.

### Where a new window first appears

Most applications create a window hidden and show it afterwards. Such a
window is moved into the slot it is going to get when it is created, before
it is shown, so it is usually in place when it appears. The move is posted
to the application and lands when its thread next reads its message queue,
so an application that shows the window before then shows it where Windows
put it first.

A window that is already visible by the time mosaico handles its creation,
for example one an application creates with `WS_VISIBLE`, is on screen
before mosaico can move it. When mosaico takes it in, it first posts the
window a move to its slot, which does not wait for the application, and then
lays the monitor out exactly as it otherwise would. The posted move lands
once the application has finished creating the window and reads its
messages, so the window no longer waits for the other windows to be moved
and their borders drawn. How long it shows where Windows put it depends on
the application:

- for a window that is not activated as it opens, usually a few
  milliseconds;
- for one that is activated as it opens, longer: about 30 ms on the
  machine measured in #28.

A window restored from minimized, or taken in later because its title or
focus changed, is moved the same way. A maximized one is left to the
layout, since only a placement takes a window out of maximized state, and
so is a window taken in on a monocle workspace: it becomes the monocle
window, which the layout places right after any window waiting behind it.

Hiding the window across the move does not help. Explorer can cloak a
window only once it has registered it, a few milliseconds after the window
is shown, and the cloak takes another millisecond or two. For a window that
is not activated, the move has usually landed by then. For one that is
activated, the cloak lands first, but in most runs the activation undoes it
1 to 3 ms later. The other ways to hide another process's window are
refused or wait for that application, and only a hook injected into every
process runs before the window is shown. The measurements are in #28.

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
  spatial algorithm differs. A split the user moved keeps a ratio of its own,
  on its workspace and for its layout, named by its place in the window order
  (split `k` divides windows `k..` into window `k` and the rest), so a swap
  keeps the sizes of the slots, and a layout that has no ratio for a split
  lays it out as the config says.
- Programmatic moves never trigger `Moved`, which only reports the user
  finishing a drag, so laying windows out cannot loop.
- Stale handle pruning runs before every layout application to handle UWP apps
  that close without firing destroy events, and windows minimized without a
  report. It leaves alone a window the tiler itself hid with `hide` and is
  about to show, which is out of view by the tiler's doing, not stale.
- `Workspace` stores a `layout_kind` alongside its `Vec<WindowId>` list
  -- the layout algorithm determines spatial positioning, not insertion order.
- The core layout module is split into sub-files (`mod.rs`, `bsp.rs`,
  `vertical_stack.rs`, `three_column.rs`, `tests.rs`) with one layout per
  file. The tiler in `crates/mosaico-core/src/tiler/` is split the same way,
  one concern per file: the state machine in `mod.rs`, `step` in `step.rs`,
  startup in `startup.rs`, actions in `actions.rs`, window events in
  `events.rs`, layout in `layout.rs` and `layout_actions.rs`, workspaces in
  `workspace.rs`, navigation in `navigation.rs`, and the tiling manager that
  runs it in `manager.rs`.
- `restore_all_windows()` iterates all workspaces on all monitors to ensure
  no window remains invisible after daemon shutdown.

## Tests

- 5 unit tests for `BspLayout`: single window, two windows, three windows,
  empty input, large gap safety
- 6 unit tests for `VerticalStackLayout`: single window, two windows, three
  windows, five windows, empty input, inner gaps that match the outer padding
- 6 unit tests for `ThreeColumnLayout`: single window, two windows, three
  windows (center master), five windows (alternating sides), empty input,
  inner gaps that match the outer padding
- 2 unit tests for `LayoutKind`: cycle order, display names
- 3 unit tests for `Workspace`: add/remove, insert at position, layout
  delegation, in `workspace_tests.rs`, 8 for its monocle state in
  `workspace/monocle_tests.rs`, and 6 for what its windows coming, going and
  trading places do to its splits in `workspace/splits_tests.rs`
- Split tests in `layout/`: each split of the BSP chain and the masters at
  their own ratio, each layout's own set, the bounds, and windows inserted
  and removed (`split_ratios_tests.rs`, `split_chain_tests.rs`); which split
  each edge of each window borders in every layout (`split_edges_tests.rs`);
  and every edge that borders one, and both edges of every corner of every
  window, dropped in every layout of two to five windows, found where they
  were dropped once laid out again, the centered master, the bounds, and
  held edges moved no more than a press wobbles, or put back on their slot
  or where they started, which move no split (`split_resize_tests.rs`,
  `split_corner_tests.rs`, `split_clamp_tests.rs`)
- Tiler tests in `crates/mosaico-core/src/tiler/tests/`, against a fake
  desktop: event handling, action dispatch, workspace switching, config
  reload, window restore
- Tiling manager tests in the same place, against a fake platform: what is
  held back until it is flushed, echoes of its own effects, the hidden-window
  record, and a restart after the daemon was killed
- Transition tests in the same place: the step decision, on made-up steps
  and on a `Tiler`, against the fake desktop of `Harness`
  (`plan_transitions*_tests.rs`, `step_transition*_tests.rs`); held
  placements keeping their mark, on the `Coalescer` alone
  (`coalesce_transition_tests.rs`); and motions, frames, endings, display
  changes, borders, posted placements and their landing, changes of show
  state, transitions turned off while windows are still posted to, the
  pointer, hover and reloads, on the tiling manager, the fake platform and
  its clock (`motion_*_tests.rs`). The motion model is tested on its own
  in `motion/tests/` (see [animations.md](animations.md#tests))
- Drag tests in the same place: where a drop lands and how a drag ended, as
  plain functions (`drag_target_tests.rs`, `drag_cancel_tests.rs`,
  `drag_ending_tests.rs`); on a `Tiler`, each drop target, a drop the
  system snapped or maximized, on its monitor or another, Escape with the
  pointer over another tile, a click, a start read late, a resize, a drag
  without position reports, a window the tiler leaves alone or does not
  manage yet, a monocle workspace, global mode, a key, the wheel over a bar
  or a display change mid-drag, a window wider than its slot, one rescaled
  across two monitors and `drag_to_swap` off (`drag_drop*_tests.rs`); what
  the tiler leaves alone while it follows a drag, where it starts, its
  border, and what ends one, reported or not (`drag_tests.rs`,
  `drag_start_tests.rs`, `drag_border_tests.rs`, `drag_end_tests.rs`,
  `drag_lost_tests.rs`); a drag and the bars: a click on the dragged
  window's own bar mid-drag, a drop over a bar, and a window dragged out of
  maximized (`drag_bar_tests.rs`); the preview, over a bar too, and the bars
  while a window is dragged (`drag_preview*_tests.rs`); and, on the tiling
  manager, what was held back for a grabbed window, the pointer while a
  window is dragged or resized and what gives it back or leaves it where
  the user has it, focus follows mouse while a window is held, a key while
  a window is held with transitions on, a drop with them on, and a click
  that leaves the border drawn (`applier_drag*_tests.rs`,
  `drag_hold_tests.rs`, `drag_drop_motion_tests.rs`,
  `manager_daemon_tests.rs`). The metrics scenarios `drop_to_swap` and
  `drop_on_another_monitor` count what a swap and a drop on another monitor
  ask of the platform
- Split tests on a `Tiler`, in the same place: what retiling, cycling
  layouts, monocle, a swap, a window opening or closing, a config reload, a
  drop on another monitor and a monitor that disconnects and comes back do
  to the splits, with windows that closed or went elsewhere while it was
  away, a window displaced twice, and its bar over a maximized window
  (`split_ratio_tests.rs`, `drag_drop_monitor_tests.rs`,
  `reconnect_split_tests.rs`, `reconnect_away_tests.rs`); a resize by an
  edge and a corner in each layout, the edges that go back, the bounds, a
  resize with `resize_splits` off or turned off by a reload, cancelled,
  interrupted by a key or by a click or the wheel on a bar, in monocle,
  without a pointer or let go over a bar, a corner dragged along one edge,
  also of a window taller than its slot, an edge that wobbled, Escape on a
  window wider than its slot, a press let go where it was, a window left
  maximized, a window gone without a report, and a swap or layouts cycled
  after a resize (`drag_resize*_tests.rs`); and, on the tiling manager with
  transitions on, a window held by its edge that nothing places, and the
  windows a resize moves sliding into their slots
  (`drag_resize_motion_tests.rs`). The metrics scenario `resize_split`
  counts what a resize asks of the platform
- `crates/mosaico/tests/drag_and_drop.rs` drags two notepads by their title
  bar on the live desktop, with the real pointer and transitions off, so it
  runs on demand with the desktop suites, never with the safe ones: one
  dropped on the other swaps places with it, and one dropped on its own
  slot goes back to it. It also drags the left one by its right edge, once
  a click on its title bar has brought it to the front, and finds the right
  one one gap after where the edge was dropped. It checks first that the
  notepads are alone and side by side on workspace 8, and otherwise fails
  saying its setup is wrong. It grabs a notepad where the notepad says its
  caption or its right edge is, the caption below the band that resizes it
  at any scale, and once a drag ends, or fails half way, the button is up
  and the pointer is back where it was
- Extensive spatial navigation tests in `spatial.rs` (see
  [monitor-management.md](monitor-management.md))

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
| `crates/mosaico-core/src/workspace.rs` | `Workspace` -- ordered collection of window handles per monitor, tracks `LayoutKind` |
| `crates/mosaico-core/src/workspace/monocle.rs` | Monocle state of a `Workspace`, with the windows a display change brought onto it that wait behind the monocle window |
| `crates/mosaico-core/src/rect.rs` | `Rect` -- rectangle type with spatial helpers |
| `crates/mosaico-core/src/spatial.rs` | Pure spatial navigation functions (`find_neighbor`, `find_entry`) |
| `crates/mosaico-core/src/tiler/mod.rs` | `Tiler` -- every tiling decision, returned as effects; `MonitorState` in `state.rs`; `Tiler::step` in `step.rs` |
| `crates/mosaico-core/src/tiler/placement.rs` | `Timing` and `Transition` -- when a placement reaches the screen, and whether it may animate |
| `crates/mosaico-core/src/tiler/plan_transitions.rs` | `Plan::place()` and the step decision that marks the placements that may animate |
| `crates/mosaico-core/src/tiler/navigation.rs` | `focus_direction()`, `move_direction()` -- spatial navigation and cross-monitor moves |
| `crates/mosaico-core/src/tiler/layout.rs`, `layout_actions.rs` | `apply_layout_on()`, `retile_all()`, `toggle_monocle()`, `cycle_layout()` -- layout application |
| `crates/mosaico-core/src/tiler/workspace.rs` | `goto_workspace()`, `send_to_workspace()` -- workspace switching |
| `crates/mosaico-core/src/tiler/manager.rs`, `manager_inputs.rs` | `TilingManager` -- runs the tiler on a `Platform`, event/action dispatch |
| `crates/mosaico-core/src/tiler/applier.rs` | Applies effects, holds focus changes and moves back, drops their echoes |
| `crates/mosaico-core/src/tiler/applier_motion.rs` | Placements that animate: the motions they become, and their ends |
| `crates/mosaico-core/src/tiler/platform.rs` | `Platform` -- what the tiling manager needs from the OS |
| `crates/mosaico-windows/src/tiling/platform.rs` | `Win32Platform` -- the Win32 implementation of each of those contracts |

### Key Types

- `Layout` (trait) -- defines `fn apply(&self, windows: &[WindowId], work_area: &Rect) -> Vec<(WindowId, Rect)>`
- `LayoutKind` (enum) -- `Bsp`, `VerticalStack`, `ThreeColumn`; provides
  `next()` for cycling and `name()` for status-bar display
- `BspLayout` -- implements `Layout` with configurable `gap` and `ratio`
- `VerticalStackLayout` -- implements `Layout`: master pane left, vertical
  stack right
- `ThreeColumnLayout` -- implements `Layout`: master pane center, stacks on
  both sides; extra windows alternate left/right
- `Workspace` -- maintains an ordered `Vec<WindowId>` of the windows on a
  monitor plus a `layout_kind: LayoutKind` field; provides `add()`,
  `insert()`, `remove()`, `swap()`, `compute_layout()`,
  `layout_kind()` / `set_layout_kind()`
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
- `Moved` -- detects manual monitor reassignment (drag to different monitor)
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
dedicated methods in the split tiler modules:

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
the step finishes, `Plan::finish` (`plan_transitions.rs`) runs the step
decision, `motion::step_animates`, over the placements of the step and marks
the ones it allows `Animate`. It needs facts only the step has:

- whether transitions are on (`[animations]` enabled, with a duration above
  zero), which the tiler takes from `Settings` at start and from every
  config reload;
- whether the step shows the window, moves it ahead of the layout
  (`PrePlace`) or takes it in (`Plan::take_in`, from `add_and_focus`,
  `try_adopt` and a window dropped onto a monitor): such a window appears
  where it goes instead of sliding there;
- whether the window is put behind another before its placement, as the
  windows waiting behind a monocle window are;
- whether the step is a display change, the tiler's start, or the bar
  reservation right after it (`Tiler::step` in `step.rs`): everything snaps
  while monitors and work areas are in flux;
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
more than 4 pixels, and the platform can move windows through frames. A
placement that passes becomes a motion in the applier's `Motions` table,
starting from where the window is as far as its motion knows if it is moving,
otherwise from where the desktop has it, with the duration and easing
configured then. The applier takes the `[animations]` settings at start and
from every reload. A motion in flight keeps the duration and easing it
started with while its target stays; one that a reload's retile sends
elsewhere turns towards its new target with the new ones, and with
transitions off it snaps there. Any other placement of a moving window ends
its motion and places it at once, in full.

The windows an `apply` or a `flush` sets moving form a step
(`Motions::begin_step`).

The frame step (`applier_frames.rs`, `TilingManager::frame()`) posts every
frame due in one `place_frames` call, and acts on what the platform found of
each window (`motion/motions_frame.rs`):

- A window that handled the frame before moves on, and its border is moved
  to where the frame found it, or, when it took the frame posted at the step
  before exactly, to the frame now posted: the border neither trails a
  window that keeps up nor runs ahead of a slow one. One still to handle its
  frame stays, and so does its border.
- A window found still to handle its frame and not seen to move for 50 ms
  (`MAX_LAG`) since it handled its first frame has fallen behind: every
  motion of its step, and no other, ends at once with a full placement at
  its target. The limit is a time, so it is the same at any refresh rate,
  and it starts at the first frame, so what ran before the first frame
  step does not count.
- A window maximized or minimized meanwhile stops where it is, and one that
  is gone is forgotten. What its motion kept for its border is dropped.
- A motion ends at its end time with a full placement at its exact target,
  as a placement without a transition is, and its border is drawn once
  there: at once, as the last frames usually took the window there, or once
  the window lands, as below.

No placement waits for a frame to land: the platform orders a placement,
and a change of how a window is shown, behind what was posted to the window
before, so a placement made while a frame is on its way lands after it. A
window whose application stopped responding after it was given frames
needs nothing of its own: its placement is posted like its frames. One that
had stopped before any was gets nothing posted, and is placed as it is
without transitions.

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
that cannot have display frames. Any other error is logged once until
frames work again, and places the windows the frames were for at their
targets, as placements that snap do.

While transitions are off, by the config, by the system's setting it
follows, or because the platform cannot move windows frame by frame, the
applier asks the platform to forget the windows it gave frames to
(`FrameControl::settle_responding`), so that changes to them are made at
once again: as they turn off, and at the start of every step after, before
the tiler reads the desktop, and of every flush, for as long as one of
those windows is still posted to, as it is within the grace or while its
application does not respond (`applier_stop.rs`). So a step decides as it
would from startup once the windows have settled.

Other effects that reach a moving window (`applier_ends.rs`):

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
- `MouseHover` is dropped while windows move. That rarely matters: a report
  takes three polls of the pointer, at least 200 ms, longer than a default
  180 ms transition, so a window sliding under a pointer at rest is usually
  reported only once it has stopped.

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
- `compute_layout(layout, work_area)` -- delegates to `Layout::apply()`
- `windows()` -- returns a slice of all the windows in the workspace

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
  spatial algorithm differs.
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
  one concern per file: the state machine and `step` in `mod.rs`, startup in
  `startup.rs`, actions in `actions.rs`, window events in `events.rs`,
  layout in `layout.rs` and `layout_actions.rs`, workspaces in
  `workspace.rs`, navigation in `navigation.rs`, and the tiling manager
  that runs it in `manager.rs`.
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
- Tiler tests in `crates/mosaico-core/src/tiler/tests/`, against a fake
  desktop: event handling, action dispatch, workspace switching, config
  reload, window restore
- Tiling manager tests in the same place, against a fake platform: what is
  held back until it is flushed, echoes of its own effects, the hidden-window
  record, and a restart after the daemon was killed
- Extensive spatial navigation tests in `spatial.rs` (see
  [monitor-management.md](monitor-management.md))

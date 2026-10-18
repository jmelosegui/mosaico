# Phase 35: Mouse Interaction

**Status:** In Progress

**Goal:** Let the mouse drive the tiler: click the bar to change workspaces,
scroll it to step through them, click its widgets, drag a tiled window onto
another to swap them or onto another monitor to move it there, and drag the
edge between two tiles to resize them.

## Overview

Today the mouse can only focus windows (focus follows mouse) and is moved by
focus changes (mouse follows focus). The bar is click-through: every bar is a
layered window with `WS_EX_TRANSPARENT`, and its contract says it "never takes
focus or pointer input". A dragged window snaps back to its slot when it is
dropped on its own monitor, and is appended to the layout of another monitor
when dropped there. Resizing a tiled window with the mouse is undone as soon
as the drag ends, by the same snap back.

This phase makes the bar take pointer input, without ever taking focus, and
turns the window drags the user already makes into layout changes. As in
phase 34, the platform only reports what the pointer does and draws; every
decision (what was clicked, which monitor it acts on, what a scroll is worth,
what a drag is, where a dropped window goes, which split an edge belongs to)
is a pure function in core with tests.

The work splits into four steps, one PR each, plus the docs and checks that
come with each of them (see Tasks). 35.3 builds on `WindowEvent::MoveStarted`
from phase 34 (34.2), and drops and resizes animate when transitions are on.

## Design Goals

- The bar runs inside the daemon, so a click becomes an input on the daemon's
  own loop, against the state the bar is showing. Nothing crosses a process
  boundary, and the bar cannot fall out of sync with the tiler.
- A click names what was clicked by identity (this bar, this workspace, this
  widget), never by its position in the drawn row, so hidden or reordered items
  can never map to the wrong workspace, and a click that arrives after the bars
  were rebuilt is dropped instead of landing on something else.
- The bar takes pointer input but never focus. A click acts on the monitor of
  the bar that was clicked, whichever monitor has focus, and an action that
  comes from the pointer never moves the pointer, without any setting being
  switched off and on around it.
- The bar never takes a click meant for a window under it: over a maximized or
  fullscreen window on its monitor it is click-through, as it is today.
- Clicks run the same actions as the keyboard, through the same path, so they
  honour the workspace mode (in global mode, clicking a workspace switches every
  monitor) and pausing.
- A drag is about the window the platform reports being dragged, never
  whichever window has focus. What a drag is (a move or a resize, and of which
  edges) is known when it starts, from where the pointer grabbed the window, and
  where it was dropped is read when it ended, not when the daemon got to it.
- While a window is dragged, a preview shows where it will land, nothing else
  moves the window or the pointer, and the window has focus and its border,
  which follows it.
- Every new kind of work the tiler does has a metrics scenario, recorded with
  the change.

## Configuration

`bar.toml`:

```toml
[mouse]
enabled = true           # false keeps the bar click-through, as before
scroll = "workspaces"    # what scrolling over the bar does: "workspaces" or "none"

# Clicks on widgets run actions, named as in keybindings. These are the
# defaults; an entry for the same widget and button replaces one.
[[mouse.click]]
widget = "layout"
button = "left"          # left, right or middle
action = "cycle-layout"

[[mouse.click]]
widget = "layout"
button = "right"
action = "toggle-monocle"

[[mouse.click]]
widget = "paused"
button = "left"
action = "toggle-pause"
```

`config.toml`:

```toml
[workspaces]
wrap = false             # stepping past the last workspace goes back to the first

[mouse]
follows_focus = true     # existing
focus_follows_mouse = false  # existing
drag_to_swap = true      # dropping a tiled window on another swaps them
resize_splits = true     # dragging the edge between two tiles resizes the split
```

The bar takes input by default. A left click on a workspace label always goes
to that workspace; it is not a binding. Click bindings may only name actions
that act on a monitor (`cycle-layout`, `toggle-monocle`, `goto-workspace-N`,
`goto-workspace-next`, `goto-workspace-prev`) or on the whole daemon
(`toggle-pause`, `retile`). Actions about the focused window (`close-focused`,
`minimize-focused`, `send-to-workspace-N`, `focus-*`, `move-*`) would act on a
window of another monitor when clicked there, so a binding that names one is
reported as invalid and ignored.

## What the Mouse Does

### On the bar

| Where | What | Result |
|-------|------|--------|
| A workspace label (left side) | left click | Go to that workspace on the bar's monitor (every monitor in global mode) |
| Anywhere on the bar | wheel | Previous (up, left) or next (down, right) workspace on the bar's monitor, with `scroll = "workspaces"` |
| The layout widget | left / right click | Cycle the layout / toggle monocle on the bar's monitor (defaults) |
| The paused widget | left click | Resume the hotkeys (default) |
| A clickable item | hover | The pointer shows a hand; elsewhere on the bar, the arrow |

- A press and a release on the same item is a click. The bar captures the
  pointer from the press to the release, and a release on another item or off
  the bar does nothing.
- Wheel steps add up, and the switch happens once, to the workspace they add up
  to, when the wheel goes quiet. Touchpads report fractions of a notch, which
  add up until they make one.
- Focus: a click on the focused monitor's bar behaves like the key (the
  workspace's last focused window takes focus, or focus parks there if it is
  empty). A click on another monitor's bar moves focus to that monitor first,
  then behaves the same. In global mode every monitor switches, and focus goes
  to the clicked monitor. The pointer stays where it is in every case.
- Pausing pauses the hotkeys only: bar clicks keep working, and the paused
  widget is how the mouse resumes them.
- The right-side workspaces pill takes no clicks. A click on a bar whose monitor
  the tiler has no state for does nothing, in global mode too.

### On windows

| Drag | Drop | Result |
|------|------|--------|
| A tiled window by its title bar | on another tile of the same monitor | The two swap places |
| A tiled window by its title bar | on another monitor | It joins that monitor's workspace before the tile under the pointer, or at the end, instead of always at the end; the tiles there make room for it while it is dragged over them (35.6) |
| A tiled window by its title bar | on its own slot, in a gap, cancelled with Esc, or snapped or maximized by the system | It goes back to its slot, as today |
| A tiled window by an edge that borders another split | anywhere | The split follows the edge (35.4), down a stack too (35.5), and the tiles beyond it follow the edge while it is dragged (35.6) |
| A tiled window by an outer edge | anywhere | It goes back, as today |
| A window the tiler does not manage but would tile | onto a monitor | It is taken in there, as today |
| A window the tiler leaves alone | anywhere | Nothing, and the tiles under it stay put |

- A drop on another monitor's workspace in monocle ends monocle there, as
  `exit_monocle` does, and the window joins before the tile under the pointer
  in its tiled layout, or at the end. The owner chose this while testing 35.6;
  until then such a drop kept the rule for windows dragged onto a monocle
  workspace: only the monocle window was placed, and nothing was put behind
  it. With `drag_to_swap` off, a drop there still joins the end in monocle.
- A `Moved` the tiler did not see start, or one without a pointer position, is
  handled as today (`reassign_monitor`).

## Architecture

### Bar pointer input

```
bar window procedure (platform) -> PointerSink(BarId, event)      press, release, wheel, leave; bar coordinates
daemon loop                     -> Task::BarPointer               ordered with hotkeys and commands, counts as activity
core: BarId generation check    -> stale events dropped           bars are rebuilt on reload and display change
core: bar::hit(regions, x, y)   -> Hit::Workspace(i) | Hit::Widget(index) | Hit::Nothing
core: scroll accumulator        -> whole steps, applied once the wheel goes quiet
daemon action path (carry_out)  -> action with an origin          { monitor, from_pointer }
```

- `BarRenderer::create` is given a `BarId` (monitor and generation, assigned by
  core) and a `PointerSink` that core defines. `BarSurface::draw` returns where
  it drew each item: the item, a workspace index for each workspace label, its
  widget's index in the config list, and its rect in bar coordinates. Fonts and
  text widths stay in the platform. The regions are empty after a failed draw
  and after `hide`, and hit testing follows paint order (the right side last,
  so it wins where the sides overlap).
- The content says which items are clickable, and the platform answers the
  pointer shape synchronously by calling core's `hit` on the regions it last
  returned.
- While input is on, the bar paints its clickable items at an alpha of at least
  1, invisible but enough for the whole of each item to take input, since a
  layered window passes input through fully transparent pixels. The rest of
  the strip keeps the alpha it is painted with, so where a transparent bar
  draws nothing, a window under it keeps its clicks.
- The bar is click-through while a maximized or fullscreen window is on its
  monitor: core decides from `Desktop` on each bar update, and the platform
  switches.
- Only press, release, wheel and leave are forwarded; pointer moves are not, so
  hovering never counts as activity. The bar's window procedure runs when the
  daemon pumps its messages, at the top of each loop iteration, so a click can
  wait up to `PUMP_INTERVAL` (50 ms) and can run after a key pressed just after
  it. That is accepted in this phase; waiting on the window message queue too is
  listed under Later.
- Actions carry an origin: the monitor they act on and whether they came from
  the pointer. Monitor-scoped actions (`cycle_layout`, `toggle_monocle`,
  `goto_workspace_on`) use it instead of `focused_monitor`, and pointer-origin
  work emits no `CenterCursor` and no `move_cursor` (`take_focus` and
  `center_cursor_on_focused`). `monitor_at_point` counts the bar strip as part
  of its monitor.
- New actions for stepping through workspaces, also bindable to keys and usable
  from the CLI: `goto-workspace-next` and `goto-workspace-prev`, with the `wrap`
  rule. `Action::from_str` matches them before parsing a number.

### Drags (`mosaico-core/src/tiler/drag.rs`)

- `MoveStarted` gains what the pointer grabbed (the title bar, or which frame
  edges), the window's rect then, whether it is maximized, and the pointer
  position, all read when the platform saw the drag start. `Moved` gains the
  pointer position read when the drag ended. `MoveStarted` and `Moved` are
  barriers in `daemon/dedup.rs`, so position reports are never merged across
  them.
- `MoveStarted` for a tiled window starts a drag for it: what was held for the
  window is dropped (as `Untile` does), nothing places the window until the drag
  ends, and pointer moves from the tiler wait until the drag ends. The window
  is the tiler's focused one from the drag's start, without asking the platform
  mid-drag, and the drop asks it when no report said the system gave the window
  focus; its border stays on it, drawn where each position report has it. The
  owner chose this while testing 35.6; until then the border was hidden once a
  report found the window away from where it started, so a click on the title
  bar left it. A drag that started on a maximized window, as the platform saw
  it then, is a move. When the platform cannot say what was grabbed, comparing
  the window's size at the start and at the end tells a move from a resize.
- While a move goes on, each position change works out the drop target from the
  pointer, and the preview follows it. Without position changes during the drag
  (window contents not shown while dragging), `Moved` decides from the final
  rect and pointer alone, without a preview.
- 35.6, added while testing 35.5 on the desktop, lays out live what the drop
  would do: each position change of a drag the tiler decides places the tiles
  beyond a held edge at the splits the drop would set, at once and without a
  transition, or the tiles of another monitor at the slots they get with the
  window put where it would join, a workspace there in monocle laid out tiled,
  as any layout change, gliding there with transitions on and turning from
  where they are as the target changes, and only those whose slot changed. The
  window itself is never placed, and nothing is committed until the drop, which
  finds them in place or on their way. Once the drag no longer lays them out,
  as it leaves, for Escape, a key, a bar action, a display change or a drop
  elsewhere, they go back where their workspace has them, a monocle workspace
  with its monocle window in front again, and they may animate back. Without
  position changes, nothing follows.
- `Moved` ends the drag and applies the drop. A final rect within a few pixels
  of the rect at the start, or equal to the window's slot, or a pointer back
  within a few pixels of where the drag started it, is a cancel or a click on
  the title bar, and changes nothing; a few pixels are what a press may wobble
  before Windows starts a drag. A drop the system turned into a snap or a
  maximize on the window's own monitor goes back to its slot; one maximized on
  another monitor joins it and stays maximized. A size a few pixels off on
  every edge, as a monitor of another scale rounds it, is no snap, and neither
  is one the system rescaled while most of the window was on another monitor.
- Every action that reaches the tiler, from a key, a command, a click or the
  wheel on a bar, and a display change, from when it is noted, interrupt a drag
  going on: the preview is hidden, nothing places the window, and the `Moved`
  that follows is handled as today. `toggle-pause`, from a key or the
  **PAUSED** mark, is the daemon's and leaves the drag alone. A workspace
  switch, a close, hide or minimize of the dragged window, and another window
  taking focus other than by the tiler's doing end the drag, as the end of a
  drag may never be reported.
- The insert position on another monitor is before the tile under the pointer,
  or at the end over an outer gap or the bar. In a gap between tiles, inside
  the box they fill, it keeps the target the last report decided (the end
  with none), so the tiles that made room do not bounce; the owner chose
  this in the #160 review, round 1. The preview shows the slot the window
  gets in that monitor's
  layout once it is inserted, which in BSP is not the tile under the pointer.

### Split resizing

- 35.4 resizes the splits that have a ratio today or get one here: each split of
  the BSP chain (split `k` divides windows `k..` into window `k` and the rest),
  the master edge of the vertical stack, and the master edges of the three
  columns. Other edges (between windows of a stack, or of a side column) go
  back; 35.5 changed that, and resizes them too.
- The ratios live with each workspace, one set per layout, and start from
  `layout.ratio`. A swap keeps them (the slots keep their sizes, the windows
  trade places); inserting a window at `k` inserts the default ratio at split
  `k`, and removing one removes its split; cycling layouts keeps each layout's
  own; monocle leaves them. A retile puts them back: the workspace on screen
  of every monitor forgets every split the user moved, in all its layouts,
  and follows `layout.ratio` again, while a workspace out of view keeps its
  own. That was chosen while testing 35.4 on the desktop, as the way back
  from moved splits; until then a retile left them alone.
- A split nobody moved has no ratio of its own and follows `layout.ratio`, so
  a reload with a new ratio still moves it. The master's ratio in the
  vertical stack and the three columns is no window's split and stays as
  windows come and go. The master of the three columns stays centered: its
  two edges are one split, and dragging either moves both. A monitor that
  disconnects and comes back gives its windows back their BSP splits.
- The new ratio is where the edge was dropped, as a fraction of the split's
  extent, clamped to 0.1 to 0.9 (core has no minimum-size contract). It is
  read from the window's rect at the end of the drag, so a resize whose end
  has no pointer moves its split all the same. A held edge dropped within a
  few pixels of its slot or of where it started moves no split, so a corner
  dragged along one edge leaves the other split as it was. A resize whose
  grab the platform could not tell, as of a window that runs its own move
  and resize loops, holds the edges read from how it changed the window: on
  each axis, the edge that moved more than a few pixels while the other
  stayed. One that moved both edges of an axis, as a move, a rescale across
  monitors or a snap does, or that leaves the window off its own monitor,
  goes back, as today.
- 35.5, added while testing 35.4 on the desktop, resizes the stacks too. The
  stack of the vertical stack, and each side column of the three columns,
  is shared equally until the user drags an edge between two of its windows,
  laid out to the pixel as before. Then it gets heights: each window has a
  share of the column's height without its gaps, the shares summing to 1,
  and each window ends where its share and those above it put it, rounded,
  the last at the bottom, so the gaps stay whole. Down a stack, a window's
  top and bottom edges border the splits between it and the windows beside
  it, but at the ends, and its corners combine them with the master's
  edges. Dragging such an edge trades height between those two windows
  only: their two shares change, keeping their sum, and every other edge
  stays where it was, to the pixel. Every window keeps a tenth of an equal
  share of its column, where the edge stops, so an edge never moves against
  the drag. A window that joins a resized stack gets an equal share, which
  the others give up in proportion, and one that leaves gives its share
  back in proportion, so none is squeezed as windows come and go. In the
  three columns, a window that comes or goes before the last side window
  moves those after it to the other column, and both columns go back to
  equal heights; one that joins after the last, or the last that leaves,
  changes only its own column. A swap, cycling layouts, monocle and reloads
  keep the heights, a retile forgets them, and a monitor that comes back
  gives its windows back their shares, so a whole monitor back in order has
  its heights back, to the pixel, while a window back in the other side
  column evens both columns, as a close at home does.
- Phase 25's keyboard resizing is rewritten against this model.

### Platform contracts

Core states every contract in behaviour terms; the platform's API names stay in
the platform's docs (the `os_names` metric counts them in core).

- `BarRenderer` / `BarSurface`: a bar takes pointer input but never focus or
  activation, is still never reported to the tiler as a window to manage, and
  reports press, release (with the button), wheel steps (as fractions of a
  notch) and leaving, with positions in bar coordinates, to its `PointerSink`.
  It can be switched to click-through. A platform that cannot take pointer input
  stays click-through and reports nothing. `draw` returns the item regions.
  Windows: no `WS_EX_TRANSPARENT` while input is on, `WS_EX_NOACTIVATE` and
  `MA_NOACTIVATE`, `SetCapture` from press to release, `WM_SETCURSOR`,
  `TrackMouseEvent`, and wheel positions converted from screen coordinates.
- `Decorations::show_preview(rect, style)` and `hide_preview()`, applied as an
  effect pair through `apply_decoration_effect`: an outline with a translucent
  fill, in its own style, drawn just behind the dragged window, in its band,
  never taking input, and rendered again only when its rect changes.
- `WindowEvent::MoveStarted` and `WindowEvent::Moved` carry the facts above.

## Edge Cases

1. Global workspace mode: a click or a scroll switches every monitor, and focus
   goes to the clicked monitor.
2. A click while windows are moving (phase 34): the action retargets them.
3. A click on the bar of a monitor whose workspace is empty: the switch happens,
   and the empty-workspace focus claim (`empty_focus.rs`) is kept.
4. A burst of wheel steps: they add up, and one switch is made once the wheel
   goes quiet.
5. Pressing on one label and releasing on another: nothing happens.
6. The bars are rebuilt between a press and its release: the release names a
   stale bar and is dropped, and press state is reset.
7. A maximized or fullscreen window on the bar's monitor: the bar is
   click-through until it is gone.
8. Dragging across monitors with different scale factors: the drop target comes
   from the pointer, not from the window's rect, which the system rescales on
   the way.
9. Esc during a drag: the system puts the window back, the final rect equals the
   start, or the slot when the start was read a little late, and nothing
   changes, even with the pointer over another tile.
10. A hotkey during a drag: nothing places the window, and its `Moved` is
    handled as today. Alt+2 hides it with its workspace and ends the drag.
11. A window opening during a drag: it is tiled, but its focus does not move the
    pointer until the drag ends.
12. A neighbour with a minimum size: the ratio is clamped to 0.1 to 0.9, and the
    retile places both windows as today. Down a stack, the floor is a tenth of
    an equal share of the column, 26 pixels for four windows stacked on 1080
    and 15 once three more open, which can be under a window's minimum
    height, so such windows can overlap, as with BSP's tenth.
13. Wheel input with "Scroll inactive windows when I hover over them" off: the
    bar may get none. 35.1 checks it, and documents the dependency if so.
14. Window contents hidden while dragging: no position reports reach the tiler
    during a drag, so no tile follows it live, and the drop moves them as
    before.

## Later

- Dropping a window on a bar's workspace label to send it there. It needs the
  item regions in the tiler in screen coordinates, the label drawn above the
  dragged window, and an answer for the system's snapping at the top edge.
- Waiting on the window message queue as well as the channel, so bar clicks
  are not delayed by `PUMP_INTERVAL`.
- Modifier + drag anywhere in a window (Alt + drag to move, Alt + right drag to
  resize), which needs a system-wide pointer hook and its latency cost.
- Media controls on the media widget, once phase 19 lands.
- Dragging a workspace label onto another monitor's bar to move the whole
  workspace there.
- Laying out the window's own monitor without it while it is dragged over
  another.

## New and Modified Files

New code goes in new files where the file it belongs next to is already near or
over the 150-line limit (`bar/window.rs`, `bar/render.rs`, `bar/manager.rs`,
`action.rs`, `config/bar.rs`, `config/types.rs`, `tiler/workspace.rs`,
`daemon_loop_run.rs`).

```
crates/
  mosaico-core/src/
    bar/renderer.rs            # pointer input, BarId, PointerSink, item regions
    bar/pointer.rs             # NEW: pointer events, hit testing, scroll accumulation
    bar/content.rs             # clickable items and their identities
    bar/manager.rs             # BarIds and generations, click-through decisions
    action.rs                  # goto-workspace-next, goto-workspace-prev, origins
    tiler/input.rs             # actions with an origin
    tiler/workspace.rs         # stepping through workspaces, from a bar's monitor
    tiler/layout_actions.rs    # monitor-scoped actions use the origin
    tiler/navigation.rs        # no CenterCursor for pointer-origin work
    tiler/empty_focus.rs       # the bar strip is part of its monitor
    tiler/drag.rs              # NEW: drag tracking and drop targets
    tiler/drag_live.rs         # NEW: the tiles a drop would move, laid out live (35.6)
    tiler/drag_live_layout.rs  # NEW: where they go, left alone under a maximized window
    tiler/drag_live_back.rs    # NEW: putting them back, what each drag sent where
    tiler/drag_aim.rs          # NEW: the target kept in a gap between tiles
    tiler/drag_mouse.rs        # the drag, its live tiles and where it sent them
    tiler/removal.rs, tiler/step.rs  # what ends a live layout
    tiler/tests/drag_live_*    # NEW: the live layout's tests
    tiler/adopt.rs             # the drop decides instead of reassign_monitor's snap back
    tiler/applier.rs, tiler/coalesce.rs  # what a drag start drops
    daemon/dedup.rs            # MoveStarted and Moved are barriers
    event.rs                   # what MoveStarted and Moved carry
    layout/, workspace.rs      # split ratios per workspace and layout
    decoration.rs              # show_preview, hide_preview
    config/bar.rs              # [mouse] in bar.toml
    config/types.rs            # [workspaces] wrap, [mouse] drag_to_swap, resize_splits
    config/template_bar.rs, config/template_config.rs, config/store/  # templates and hints
    tiler/tests/               # fakes, metrics scenarios
  mosaico-windows/src/
    bar/window.rs              # pointer input without activation, window procedure
    bar/render.rs              # item regions, minimum alpha
    event.rs                   # what MoveStarted and Moved carry
    daemon_types.rs            # DaemonMsg::BarPointer
    daemon_loop.rs, daemon_loop_task.rs, daemon_loop_run.rs, daemon_loop_commands.rs
    decorations.rs, border/    # the drop preview overlay
  mosaico/src/main.rs          # the new actions in the CLI
  mosaico/tests/               # integration tests for a bar click and a drop
metrics/                       # baselines recorded with each step
docs/
  status-bar.md, configuration.md, window-management.md, actions.md, daemon.md,
  workspaces.md, tiling-layout.md, keyboard-bindings.md, cli.md
website/src/guide/
  status-bar.md, configuration.md, keybindings.md, workspaces.md, tiling-layout.md, cheatsheet.md
```

## Tasks

Every step runs fmt, clippy, the safe test suites and `cargo xtask metrics`,
records its metrics and scenarios with `cargo xtask metrics --bless`, and brings
its docs, templates and config hints, so it can merge on its own. 35.1 and 35.2
need nothing from phase 34; 35.3 needs `MoveStarted` (34.2); 35.4 needs 35.3,
and 35.5 needs 35.4.

### 35.1 Bar clicks and scrolling

- [x] `BarId`, `PointerSink` and item regions in the bar contracts, and
      click-through over maximized or fullscreen windows
- [x] Hit testing in core against the drawn regions, with tests for every item
      kind, the gaps, paint order, duplicate widgets, stale bars and a redraw
      between press and release
- [x] Wheel steps added up and applied once the wheel goes quiet, with tests for
      notches, fractions, direction changes and bursts
- [x] Actions with an origin: workspace clicks and scrolls act on the bar's
      monitor, focus per the rules above, no pointer moves; global mode
- [x] `goto-workspace-next` and `goto-workspace-prev`, with `[workspaces] wrap`,
      in keybindings and the CLI
- [x] Windows: input without activation, capture, the cursor, minimum alpha,
      click-through switching; tests on hidden bar windows of the test's own
- [x] `[mouse] enabled` and `scroll` in bar.toml, templates and hints
- [x] Metrics scenarios: a click on the focused and on another monitor's bar, and
      a scroll burst
- [x] Check wheel input with "Scroll inactive windows when I hover over them"
      off, and document what it needs

### 35.2 Widget clicks

- [x] `[[mouse.click]]` bindings with the defaults above, merged per widget and
      button, and validated against the actions a bar may run
- [x] Clicks on the layout and paused widgets, and on any widget a binding
      names, through the daemon's action path
- [x] Tests for the defaults, overrides, unknown widgets, rejected actions and
      clicks on another monitor's bar; a metrics scenario for a widget click

### 35.3 Drag and drop

- [x] What `MoveStarted` and `Moved` carry, and the dedup barriers
- [x] Drag tracking keyed on the dragged window, with what a drag start drops and
      holds, and what ends a drag
- [x] Drop targets: swap, insert before the tile under the pointer on another
      monitor, snap back; cancels and system snaps
- [x] The drop preview contract, effects and Windows overlay
- [x] `[mouse] drag_to_swap`, templates and hints
- [x] Tests: each drop target, Esc with the pointer over another tile, a drag
      without position reports, an unmanaged window dragged over tiles,
      adoption on drop, a monocle workspace, global mode, a hotkey and a display
      change mid-drag; metrics scenarios for a swap and a drop on another monitor
- [x] Integration test for a drop

### 35.4 Split resizing

- [x] Split ratios per workspace and layout, with what swap, insert, remove,
      layout cycling, monocle and retile do to them
- [x] Edges that resize map to their split; corners move two; the others go back
- [x] `[mouse] resize_splits`, templates and hints
- [x] Tests for every resizable edge and corner in every layout, outer edges,
      clamping, and the operations above; a metrics scenario for a resize
- [x] Rewrite phase 25 against the model

### 35.5 Stack heights

Added while testing 35.4 on the desktop, as a pull request of its own.

- [x] Heights for the windows of the vertical stack and of each side column
      of the three columns, shared equally where nobody resized them and
      laid out to the pixel as before then, with a floor of a tenth of an
      equal share
- [x] What windows coming, going and swapping, cycling layouts, monocle,
      reloads, reconnects, drops from another monitor and retile do to them,
      with the rule of the three columns for windows that change column
- [x] The top and bottom edges of a window of a stack or a column trade
      height between it and the window beside them alone; corners combine
      them with the master's
- [x] The `resize_splits` comments in the template and the hint
- [x] Tests for every edge and corner in every layout of two to five windows,
      and down stacks of up to seven with every gap, from resized heights
      too, that every other window keeps its slot and the edge stops at the
      floor, the layouts nobody resized against the code they had before,
      the heights as windows come, go and come back, random states that keep
      them sound, windows opening after a resize, edges that never go against
      the drag, and resizes down a stack in the tiler, by a window that
      reports no grab too
- [x] Up and down for the windows of a stack in phase 25's plan
- [ ] Manual test: resize the windows of a stack in the vertical stack and in
      both columns of the three columns

### 35.6 Live resize and live drop

Added while testing 35.5 on the desktop, as a pull request of its own.

- [x] The tiles beyond the edges of a resize, laid out live on each report at
      the splits the drop would set, held edges, bounds, the floor and the
      trade down a stack included, for a window that reports no grab too
- [x] The tiles of another monitor, laid out live with the window put where it
      would join, before the tile under the pointer or at the end, by the
      insertion, split and stack-height rules of the drop
- [x] The tiles beyond an edge placed at once, never animated, and those of
      another monitor as any layout change, gliding with transitions on, as
      the owner asked while testing, only when their slot changes, at most
      once a pass of the daemon loop, the window itself never placed and
      nothing committed until the drop, which finds them in place or on
      their way
- [x] Put back, with a transition if due, once the drag no longer lays them
      out: as it leaves, for Escape, a key, a bar action, a display change, a
      drop elsewhere or another drag, and by a layout of their monitor
      meanwhile
- [x] A drop on another monitor in monocle ends monocle there, as the owner
      chose while testing: laid out tiled live, put back with its monocle
      window in front, and tiled at the drop
- [x] The dragged window keeps its border, drawn where each report has it,
      through what interrupts its drag, and is the focused window from its
      start, its border in the focused or the monocle style and the one it
      left unfocused, without the platform asked mid-drag; the drop focuses it
      when no report said the system did, as the owner chose while testing
- [x] Tests for each layout, a corner, a stack, a side column, the floor,
      Escape, a key, a bar click, a display change, the options off, monocle,
      no grab, many reports, a report that moves nothing, the bar strip,
      stacks there, a window minimized without a report, one maximized at
      the drop, transitions on and drops that move only the window; a check
      while the edge is held in the desktop resize test; metrics scenarios
      for a resize with several reports and a drag that changes its target
      twice
- [ ] Manual test: drag the edge between two tiles, down a stack and to the
      floor, and watch the tiles beyond it follow, then Esc
- [ ] Manual test: drag a window over another monitor, tiled and in monocle,
      over its tiles and its bar, back over its own, with Esc and a key, and
      drop it there, with transitions on and off; cross the gap between two
      tiles there and check nothing bounces; drag over a monitor with a
      maximized window and check only the preview shows; in a stack, check
      whether the open slot jumping under the pointer is acceptable
- [ ] Manual test: grab a window that has no focus, by its title bar and by an
      edge, and watch it take focus and keep its border through the drag

### Verification

- [x] Manual test: click and scroll the bar on every monitor, in both workspace
      modes, over a maximized window, and with the bar background transparent
- [x] Manual test: drag to swap, to another monitor, cancel with Esc, drop at the
      top edge with Snap on, and with window contents hidden while dragging
- [x] Manual test: resize splits from every resizable edge in every layout
- [x] Integration test for a bar click
- [ ] Update `.plans/plan.md`

# Phase 35: Mouse Interaction

**Status:** Pending

**Goal:** Let the mouse drive the tiler: click the bar to change workspaces,
scroll it to step through them, click its widgets, drag a tiled window onto
another to swap them or onto a workspace to send it there, and drag the edge
between two tiles to resize them.

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
decision (what was clicked, what a scroll is worth, where a dropped window
goes, which split an edge belongs to) is a pure function in core with tests.

The work splits into five steps, one PR each (see Tasks). Steps 35.3 and 35.4
use `WindowEvent::MoveStarted` from phase 34 (34.2), and drops and resizes
animate when transitions are on.

## Design Goals

- The bar runs inside the daemon, so a click becomes an input to the tiler on
  the same loop, against the state the bar is showing. Nothing crosses a
  process boundary, and the bar cannot fall out of sync with the tiler.
- A click names what was clicked by identity (this workspace, this widget),
  never by its position in the drawn row, so hidden or reordered labels can
  never map to the wrong workspace.
- The bar takes pointer input but never focus: clicking it leaves the focused
  window focused. An action that comes from the pointer never moves the
  pointer, without any setting being switched off and on around it.
- Clicks run the same actions as the keyboard, so they honour the workspace
  mode: in global mode, clicking a workspace switches every monitor.
- A drag is about the window the platform reports being dragged, never
  whichever window has focus. Windows the tiler does not manage are left
  alone, and so are the tiled windows under them.
- Whether a drag moves or resizes a window is known from its first position
  change, not guessed when it ends, and the edges that moved say which split
  changes. All four edges and the corners work.
- While a window is dragged, a preview shows where it will land.
- With the new options at their defaults for existing configs, nothing
  changes for keyboard users, and every existing test passes unchanged.

## Configuration

```toml
[bar.mouse]
enabled = true           # false keeps the bar click-through, as before
scroll = "workspaces"    # what scrolling over the bar does: "workspaces" or "none"
wrap = false             # scrolling past the last workspace goes back to the first

# Clicks on widgets run actions, named as in keybindings. These are the
# defaults; a click entry for the same widget and button replaces one.
[[bar.mouse.click]]
widget = "layout"
button = "left"          # left, right or middle
action = "cycle-layout"

[[bar.mouse.click]]
widget = "layout"
button = "right"
action = "toggle-monocle"

[[bar.mouse.click]]
widget = "paused"
button = "left"
action = "toggle-pause"

[mouse]
follows_focus = true     # existing
focus_follows_mouse = false  # existing
drag_to_swap = true      # dropping a tiled window on another swaps them
resize_splits = true     # dragging the edge between two tiles resizes the split
```

A left click on a workspace label always goes to that workspace; it is not a
binding. Scrolling one notch steps one workspace. Touchpads report fractions
of a notch, which add up until they make one.

## What the Mouse Does

### On the bar

| Where | What | Result |
|-------|------|--------|
| A workspace label | left click | Go to that workspace on the bar's monitor (every monitor in global mode) |
| Anywhere on the bar | scroll | Previous or next workspace on the bar's monitor, with `scroll = "workspaces"` |
| The layout widget | left / right click | Cycle the layout / toggle monocle (defaults) |
| The paused widget | left click | Resume the hotkeys (default) |
| A clickable item | hover | The pointer shows a hand |
| A workspace label, while dragging a window | drop | Send the window to that workspace (35.3) |

A press and a release on the same item is a click. A press on one item and a
release elsewhere does nothing, the way buttons work.

### On windows

| Drag | Drop | Result |
|------|------|--------|
| A tiled window by its title bar | on another tile of the same monitor | The two swap places |
| A tiled window by its title bar | on another monitor | It joins that monitor at the slot under the pointer, not at the end |
| A tiled window by its title bar | on a bar's workspace label | It goes to that workspace |
| A tiled window by its title bar | on its own slot, in a gap, or cancelled with Esc | It goes back to its slot |
| An edge of a tiled window that borders another tile | anywhere | The split between them follows the edge |
| An edge on the outside of the layout | anywhere | It goes back, as today |
| A window the tiler does not manage | anywhere | Nothing, and the tiles under it stay put |

## Architecture

### Bar pointer input

```
bar window procedure (platform)   -> BarPointer { bar, x, y, kind }     pointer events in bar coordinates
daemon loop                       -> Task::BarPointer                   ordered with hotkeys and commands
core: bar::hit(regions, x, y)     -> Hit::Workspace(index) | Hit::Widget(kind) | Hit::Nothing
core: bar::Scroll accumulator     -> whole notches
tiler input                       -> goto_workspace_on(bar's monitor, n), or the bound action
```

- `BarSurface::draw` returns where it drew each item: the item, a workspace
  index for each workspace label, and its rect in bar coordinates. Fonts and
  text widths stay in the platform, and core hit-tests against what was really
  drawn, so a click can only name something on screen.
- Pointer events carry the bar they came from, which belongs to one monitor,
  so a click acts on that monitor without looking at where focus is.
- Actions from the pointer are marked as such, the way hover focus is today
  (`take_focus` already leaves the pointer alone for focus that came from the
  mouse), so mouse follows focus never pulls the pointer off the bar.
- New actions for stepping through workspaces, also bindable to keys:
  `goto-workspace-next` and `goto-workspace-prev`, with the `wrap` rule.

### Drags (`mosaico-core/src/tiler/drag.rs`)

- `MoveStarted` for a tiled window starts a drag for that window, with the rect
  it had in the layout.
- Its first `LocationChanged` says whether it is a move (the size stayed) or a
  resize (it changed), and for a resize which edges moved.
- While a move goes on, each position change reads the pointer and works out
  the drop target: a tile, another monitor's slot, a bar's workspace label, or
  its own slot. The preview follows the target.
- `Moved` ends the drag and applies the drop: a swap, an insert on another
  monitor, a send to a workspace, a split resize, or a snap back.
- A window that is not tiled starts no drag, so nothing it is dropped on moves.

### Split resizing

- An edge that moved maps to the split it borders, for each of the four edges,
  and corners move two splits.
- The new ratio is where the edge was dropped, as a fraction of the split's
  extent, clamped to the ratio limits `validate` uses today (0.1 to 0.9).
- This needs a ratio per split. Phase 25 plans one for keyboard resizing. If it
  has not landed, 35.4 adds the per-split ratio model, built so phase 25's
  keyboard actions can use it unchanged.

### Platform contracts

- `BarRenderer` / `BarSurface`: a bar takes pointer input but never focus or
  activation, is still never reported to the tiler as a window to manage, and
  reports pointer events on itself: presses and releases with the button,
  wheel steps as fractions of a notch, the pointer leaving it, and positions in
  bar coordinates. `draw` returns the item regions. Windows: no
  `WS_EX_TRANSPARENT`, `WS_EX_NOACTIVATE` and `MA_NOACTIVATE`, the mouse
  messages in the bar's window procedure, `WM_SETCURSOR` for the hand, and
  `TrackMouseEvent` for leaving.
- `Decorations::show_preview(rect, style)` and `hide_preview()`: a translucent
  outline of where a dragged window will land, drawn like a border. Windows: a
  layered overlay, as for borders.
- `Desktop::cursor` already gives the pointer position during a drag.

## Edge Cases

1. Global workspace mode: a click or a scroll switches every monitor, as the
   keyboard does.
2. A click while windows are moving (phase 34): the action retargets them.
3. Clicking the bar of a monitor whose workspace is empty: the switch happens,
   and the empty-workspace focus claim (`empty_focus.rs`) is kept.
4. Scrolling fast with a touchpad: fractions add up, and one input never steps
   more than the notches it carried.
5. Pressing on one label and releasing on another: nothing happens.
6. The bar is redrawn between a press and its release: the release is
   hit-tested against the new regions, and a click needs both to name the same
   item.
7. Dragging across monitors with different scale factors: the drop target
   comes from the pointer, not from the window's rect, which the system
   rescales on the way.
8. Dropping onto a workspace in monocle: the window waits behind the monocle
   window, the rule for windows arriving there.
9. A drag the user cancels with Esc: the system puts the window back and ends
   the drag, and the drop is a snap back.
10. A neighbour with a minimum size: the ratio is clamped, and the retile
    places both windows as today.
11. The tiler moving a window while the user drags it: no placement is applied
    to a window being dragged; the drop decides.

## Later

- Modifier + drag anywhere in a window (Alt + drag to move, Alt + right drag to
  resize), which needs a system-wide pointer hook and its latency cost.
- Media controls on the media widget, once phase 19 lands.
- Dragging a workspace label onto another monitor's bar to move the whole
  workspace there.
- Neighbours following an edge while it is dragged, instead of on the drop.

## New and Modified Files

```
crates/
  mosaico-core/src/
    bar/renderer.rs            # pointer input and item regions in the contract
    bar/pointer.rs             # NEW: pointer events, hit testing, scroll accumulation
    bar/content.rs             # item identities for hit testing
    action.rs                  # goto-workspace-next, goto-workspace-prev
    tiler/workspace.rs         # stepping through workspaces, from a bar's monitor
    tiler/drag.rs              # NEW: drag tracking and drop targets
    tiler/adopt.rs             # the drop decides instead of reassign_monitor's snap back
    layout/                    # per-split ratios (with phase 25)
    decoration.rs              # show_preview, hide_preview
    config/bar.rs              # [bar.mouse]
    config/types.rs            # [mouse] drag_to_swap, resize_splits
  mosaico-windows/src/
    bar/window.rs              # pointer input without activation, window procedure
    bar/render.rs              # item regions
    daemon_types.rs            # DaemonMsg::BarPointer
    decorations.rs, border/    # the drop preview overlay
docs/
  status-bar.md, configuration.md, window-management.md, actions.md
website/src/guide/
  status-bar.md, configuration.md
```

## Tasks

### 35.1 Bar clicks and scrolling

- [ ] Bar contract: pointer input without focus or activation, pointer events,
      item regions returned by `draw`
- [ ] Hit testing in core against the drawn regions, with tests for every item
      kind, the gaps between items, and a redraw between press and release
- [ ] Scroll accumulation in core, with tests for notches, fractions and
      direction changes
- [ ] Workspace click and scroll reach the tiler for the bar's monitor; global
      mode switches every monitor; the pointer does not move
- [ ] `goto-workspace-next` and `goto-workspace-prev` actions, with `wrap`
- [ ] Windows: the bar window takes pointer input without activating, reports
      the events, and draws the hand over clickable items; tests on hidden bar
      windows of the test's own
- [ ] `[bar.mouse]` `enabled`, `scroll` and `wrap`

### 35.2 Widget clicks

- [ ] `[[bar.mouse.click]]` bindings with the defaults above, merged per widget
      and button, and validated against the action names
- [ ] Clicks on the layout and paused widgets, and on any widget a binding
      names
- [ ] Tests for the defaults, overrides, unknown widgets and actions

### 35.3 Drag and drop

- [ ] Drag tracking from `MoveStarted` to `Moved`, keyed on the dragged window,
      with move or resize known from the first position change
- [ ] Drop targets: swap, insert at the slot under the pointer on another
      monitor, send to a bar's workspace label, snap back
- [ ] The drop preview contract and its Windows overlay
- [ ] No placement applied to a window while it is dragged
- [ ] Tests: each drop target, an unmanaged window dragged over tiles, Esc,
      a monocle workspace, global mode

### 35.4 Split resizing

- [ ] Per-split ratios (with phase 25, or added here)
- [ ] An edge that moved maps to its split; corners move two
- [ ] Tests for all four edges and the corners, outer edges, clamping, and
      every layout

### 35.5 Verification and docs

- [ ] Docs: `docs/status-bar.md`, `docs/configuration.md`,
      `docs/window-management.md`, `docs/actions.md`, and the website guide
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run the safe test suites from the testing skill
- [ ] Manual test: click and scroll the bar on every monitor, in both workspace
      modes, and check that focus stays where it was
- [ ] Manual test: drag to swap, to another monitor, onto a bar label, and
      cancel with Esc
- [ ] Manual test: resize splits from every edge in every layout
- [ ] Update `.plans/plan.md`

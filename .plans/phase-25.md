# Phase 25: Interactive Window Resizing

**Status:** Pending

**Goal:** Resize the splits of a layout from the keyboard: a key moves an
edge of the focused window, and the split that edge borders follows, on the
same splits that dragging an edge with the mouse moves since phase 35.

## Overview

Phase 35 (35.4) gave every split of a workspace's layouts a ratio of its
own, and moves a split when the user drags the edge between two tiles. This
phase adds the keyboard to the same model. Nothing new is stored: a key sets
the ratios a drag sets, through the same core functions, so a split moved
from the keyboard and one moved with the mouse are the same thing, and
everything 35.4 says of splits holds for both.

```
resize-right on window 1:

+-----------+-----------+        +--------------+--------+
|           |     2     |        |              |   2    |
|     1     +-----------+   ->   |      1       +--------+
|           |     3     |        |              |   3    |
+-----------+-----------+        +--------------+--------+

resize-up on window 3 (its bottom edge borders no split, so its top
edge, at split 1, moves up):

+-----------+-----------+        +-----------+-----------+
|           |     2     |        |           |     2     |
|     1     +-----------+   ->   |     1     +-----------+
|           |     3     |        |           |           |
|           |           |        |           |     3     |
+-----------+-----------+        +-----------+-----------+
```

## The Model

What 35.4 built, which this phase uses as it is:

- Split `k` divides windows `k..` into window `k` and the rest. Each split
  of the BSP chain is one, side by side for even `k` and one above the
  other for odd `k`. In VerticalStack and ThreeColumn, split 0, at the
  master's edges, is the only one, and the ThreeColumn master stays
  centered, so both its edges move together.
- Each `Workspace` keeps a `SplitRatios` (`layout/split_ratios.rs`), one set
  per layout, with the ratio of each split the user moved; every other
  split is at `[layout] ratio`. A ratio is kept within `MIN_SPLIT_RATIO` and
  `MAX_SPLIT_RATIO` (0.1 and 0.9). `Workspace::set_split_ratio` moves a
  split of the layout on screen.
- `split_edges::border` (`layout/split_edges.rs`) says which split an edge
  of a window borders, and how: in BSP, window `k`'s far edge borders split
  `k` unless it is the last window, and its near edges splits `k - 1` and
  `k - 2`; in VerticalStack, the master's right edge and the left edge of
  every window of the stack; in ThreeColumn, the master's two edges and the
  inner edge of every window of a side column. Other edges border none.
- `resized_splits` (`layout/split_resize.rs`) turns edges at a new place
  into ratios, to the pixel and within the bounds, and leaves alone an edge
  moved 4 pixels or less from its slot or from where it started.
- A window inserted at `k` gets a split at `ratio` at split `k`, one removed
  takes its split with it, a swap keeps every split where it is, cycling
  layouts finds each layout's own, monocle and retile leave them, and a
  config reload with a new `ratio` moves only the splits nobody moved.

## Configuration

### Keybindings

The resize actions have no default keys, as `goto-workspace-next` and
`goto-workspace-prev` have none. On a keyboard layout with no
AltGr letters on H, J, K and L, these mirror `Alt+H/J/K/L` (focus) and
`Alt+Shift+H/J/K/L` (move), and the docs suggest adding them to
`keybindings.toml` by hand:

```toml
# Resize: Alt + Ctrl + H/J/K/L
[[keybinding]]
action = "resize-left"
key = "H"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-down"
key = "J"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-up"
key = "K"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-right"
key = "L"
modifiers = ["alt", "ctrl"]
```

They are not defaults because Ctrl+Alt+key also fires for AltGr+key, which
types a letter on several layouts: on Polish, AltGr+L is the l with stroke,
and a default would take it from those users. Holding a resize key steps
once, as hotkeys do not repeat: each step is a press. `reset-splits` has no
default key either.

### `config.toml`

```toml
[layout]
# How far a resize key moves an edge, as a share of its split's area.
resize_step = 0.05
```

`resize_step` (default 0.05, clamped to 0.01 to 0.5) is reloaded with the
rest of `[layout]`; a reload keeps the ratios.

## What a Key Does

`resize-<direction>` acts on the focused window, tiled on the workspace on
screen of the focused monitor:

1. `resize-left` and `resize-right` take the window's right edge when it
   borders a split (`split_edges::border`), and otherwise its left edge;
   `resize-up` and `resize-down` take its bottom edge when it borders a
   split, and otherwise its top edge. When neither edge on that axis borders
   a split, nothing happens.
2. The edge moves `resize_step` of its split's area in the key's direction,
   from where the layout has it, and never less than 5 pixels, as a move of
   4 or less is a press's wobble to `resized_splits`, which then gives the
   new ratio, as for an edge the user dropped there. So the boundary moves
   the way the key points, whichever of the two windows beside it has
   focus, and every window can both grow and shrink on an axis it has a
   split on. The ThreeColumn master grows on both sides with
   `resize-right`, which moves its right edge right, and shrinks with
   `resize-left`, its left edge following as the master stays centered. A
   split stops at the same bounds as with the mouse.
3. The monitor is laid out again with `Timing::Coalesced`, like a move, so
   a burst of presses ends in one layout pass, and the placements may
   animate. A press that changes no ratio, as at a bound, lays nothing out
   and sets nothing. It compares the ratio in effect, the configured one for
   a split that has none of its own, so a split that follows `ratio` and
   already sits at the bound it is pressed toward keeps following it.

| Layout | Window | Left / right | Up / down |
|--------|--------|--------------|-----------|
| BSP, two windows | either | split 0 | nothing |
| BSP, three windows | 1, the left half | split 0 | nothing |
| BSP, three windows | 2 and 3, the right half | split 0 | split 1 |
| BSP, four windows | 3, below 2, left of 4 | split 2 | split 1 |
| BSP, five windows | 4, above 5, right of 3 | split 2 | split 3 |
| VerticalStack | any | split 0 | nothing |
| ThreeColumn, two windows | either, as in VerticalStack | split 0 | nothing |
| ThreeColumn, three or more | the master | split 0: right widens it on both sides, left narrows it | nothing |
| ThreeColumn, three or more | a window of a side column | split 0, at its inner edge | nothing |

`reset-splits` puts every split of the layout on screen of the focused
monitor's workspace back at `ratio`, which neither retile nor monocle does.

## Architecture

- `Action::Resize(Direction)` and `Action::ResetSplits`, named `resize-left`,
  `resize-right`, `resize-up`, `resize-down` and `reset-splits` in
  `Action::from_str` and `Display`, so keybindings, IPC and
  `mosaico action` get them. In `Action::scope()` (35.2), `Resize` acts on
  the focused window, so a bar click binding may not name it, and
  `ResetSplits` on a monitor, so one may, as for `cycle-layout`: a click on
  a bar resets the splits of that bar's monitor.
- `layout/split_step.rs`: `stepped_splits(kind, gap, slots, index,
  direction, step)`, a pure function that picks the edge as above, moves it
  from the window's slot and calls `resized_splits` with that slot as the
  start, not the window's rect: the window may still be sliding from the
  press before, and its rect would put the edge where it happens to be on
  the way. Unit tests for every window, edge and direction in every layout
  of one to five windows, the bounds and the centered master.
- `SplitRatios::reset(kind)` forgets the ratios of one layout. It refills
  the BSP chain with `None` and keeps its length, one split for each window
  but the last: cleared, the chain would be shorter than that, `insert`
  would clamp to it and `set` would ignore the splits it lacks, so some
  edges would silently stop resizing.
- `tiler/resize_actions.rs`: `resize_focused(plan, direction)` and
  `reset_splits(plan)`, dispatched from `actions.rs`.
- `LayoutConfig` moves to a file of its own first, as `config/types.rs` is
  over the size limit, and gets `resize_step`. New code in `action.rs`,
  also over the limit, goes in a file next to it.

## Edge Cases

1. A single window, an empty workspace, or no focused tiled window:
   nothing.
2. Monocle: nothing, as no edge borders a split there, as with the mouse.
3. A direction with no split on either side of the window, as up in a BSP of
   two windows: nothing, and no layout pass.
4. A split at a bound: the ratio stays, and the press lays nothing out.
5. A burst of presses: each starts from the ratio the one before set, and
   the placements are held and coalesced as for a burst of moves.
6. A key while the user drags a window: the resize interrupts the drag, as
   every action that reaches the tiler does (35.3), the split moves, and
   the dragged window takes its new slot when it is dropped. `toggle-pause`,
   from a key or the **PAUSED** mark, is the daemon's and leaves the drag
   alone.
7. Global workspace mode: only the focused monitor's workspace changes;
   splits are per workspace.
8. A layout change, a swap, a window opening or closing after a resize: as
   35.4 says.

## Tasks

- [ ] `Action::Resize(Direction)` and `Action::ResetSplits`, with
      `from_str`, `Display`, `scope()` and the CLI
- [ ] No default keys: the docs suggest `Alt+Ctrl+H/J/K/L` to add by hand,
      and say why they are not defaults and that a held key steps once
- [ ] `[layout] resize_step`, validated, in the template and with a hint
- [ ] `stepped_splits` and `SplitRatios::reset` in core, with unit tests
- [ ] The tiler actions, with tests for each direction, the opposite edge,
      nothing to move, monocle, the bounds, a burst, a key mid-drag and a
      reset
- [ ] Metrics scenarios for one press and a burst of nine
- [ ] Docs: `tiling-layout.md`, `keyboard-bindings.md`, `actions.md`,
      `configuration.md`, `cli.md`, and the website guide
- [ ] `reset-splits` wherever a text lists what a bar click may run and
      says no other action can be bound: the bar template
      (`template_bar.rs`), its hint for an existing `bar.toml`
      (`merge_bar_mouse.rs`), which stays in users' files,
      `status-bar.md`, `configuration.md`, the guide's status bar page,
      and the action names `template_mouse_tests.rs` checks. The new hint
      names what may be bound by scope, actions on a monitor or the whole
      daemon, not as a closed list, so it stays true as actions are added
- [ ] Manual test: resize every split of every layout from the keyboard, a
      burst, and a reset
- [ ] Update `.plans/plan.md`

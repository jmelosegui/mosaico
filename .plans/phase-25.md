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

What 35.4 and 35.5 built, which this phase uses as it is:

- Split `k` divides windows `k..` into window `k` and the rest. Each split
  of the BSP chain is one, side by side for even `k` and one above the
  other for odd `k`. In VerticalStack and ThreeColumn, split 0 is at the
  master's edges, and the ThreeColumn master stays centered, so both its
  edges move together. The vertical stack's stack and each side column of
  the three columns are shared equally until the user resizes one, which
  then has heights: a share of the column's height without its gaps for
  each window, summing to 1, each at least a tenth of an equal share.
- Each `Workspace` keeps a `SplitRatios` (`layout/split_ratios.rs`), one set
  per layout, with the ratio of each split the user moved and the heights
  of each stack the user resized; every other split is at `[layout] ratio`,
  and every other stack shared equally. A ratio is kept within
  `MIN_SPLIT_RATIO` and `MAX_SPLIT_RATIO` (0.1 and 0.9).
  `Workspace::set_split_ratios` puts on the workspace the splits a resize
  gives, as `move_splits` does with those `resized_splits` returns;
  `set_split_ratio` and `set_heights` are seams for tests.
- `split_edges::border` (`layout/split_edges.rs`) says which split an edge
  of a window borders, and how: in BSP, window `k`'s far edge borders split
  `k` unless it is the last window, and its near edges splits `k - 1` and
  `k - 2`; in VerticalStack, the master's right edge and the left edge of
  every window of the stack; in ThreeColumn, the master's two edges and the
  inner edge of every window of a side column; and down a stack or a
  column, a window's top edge the split between it and the window above
  and its bottom edge the one below, but at the ends. Outer edges border
  none.
- `resized_splits` (`layout/split_resize.rs`) turns edges at a new place
  into the splits and heights they give, to the pixel and within the
  bounds, and leaves alone an edge moved 4 pixels or less from its slot or
  from where it started. It takes the windows' slots and the splits they
  are laid out with, as `Tiles`, and returns the splits once the edges are
  dropped, or none when they move nothing. Down a stack or a column, an
  edge trades height between the two windows beside it alone
  (`layout/stack_edge.rs`): their two shares change, keeping their sum, so
  every other edge stays where it is, and the edge stops where either
  window would have less than a tenth of an equal share of the column.
- A window inserted at `k` gets a split at `ratio` at split `k`, and an
  equal share of a resized stack, which the others there give up in
  proportion; one removed takes its split with it and gives its share back
  in proportion; in the three columns, one that comes or goes before the
  last side window sends both columns back to equal heights. A swap keeps
  every split and height where it is, cycling layouts finds each layout's
  own, monocle leaves them, a retile forgets every split and height of the
  workspaces on screen, and a config reload with a new `ratio` moves only
  the splits nobody moved.

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
once, as hotkeys do not repeat: each step is a press.

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
   `resize-left`, its left edge following as the master stays centered.
   Down a stack or a column, a press moves the edge by a step and trades
   height between the two windows beside it alone, as the mouse does, with
   the same floor, a tenth of an equal share of the column; every other
   edge stays where it is. A split stops at the same bounds as with the
   mouse.
3. The monitor is laid out again with `Timing::Coalesced`, like a move, so
   a burst of presses ends in one layout pass, and the placements may
   animate. A press that moves no edge, as at a bound, lays nothing out and
   sets nothing. For a split that has no ratio of its own, it compares the
   configured one, so a split that follows `ratio` and already sits at the
   bound it is pressed toward keeps following it. A press on a stack nobody
   resized always moves its edge, as an equal share is ten times the floor,
   and gives the stack heights.

| Layout | Window | Left / right | Up / down |
|--------|--------|--------------|-----------|
| BSP, two windows | either | split 0 | nothing |
| BSP, three windows | 1, the left half | split 0 | nothing |
| BSP, three windows | 2 and 3, the right half | split 0 | split 1 |
| BSP, four windows | 3, below 2, left of 4 | split 2 | split 1 |
| BSP, five windows | 4, above 5, right of 3 | split 2 | split 3 |
| VerticalStack | the master | split 0 | nothing |
| VerticalStack, two windows | the other | split 0 | nothing |
| VerticalStack, three or more | a window of the stack | split 0 | its own split down the stack, or the one above it for the last |
| ThreeColumn, two windows | either, as in VerticalStack | split 0 | nothing |
| ThreeColumn, three or more | the master | split 0: right widens it on both sides, left narrows it | nothing |
| ThreeColumn, three or more | a window of a side column | split 0, at its inner edge | its own split down the column, or the one above it for the last; nothing for a window alone there |

There is no `reset-splits` action: a retile already puts every split of
the workspaces on screen back at `ratio`, and every stack back to equal
heights (35.4, 35.5).

## Architecture

- `Action::Resize(Direction)`, named `resize-left`, `resize-right`,
  `resize-up` and `resize-down` in `Action::from_str` and `Display`, so
  keybindings, IPC and `mosaico action` get it. In `Action::scope()`
  (35.2), it acts on the focused window, so a bar click binding may not
  name it.
- `layout/split_step.rs`: `stepped_splits(tiles, index, direction, step)`,
  with the `Tiles` that `resized_splits` takes, a pure function that picks
  the edge as above, moves it from the window's slot and calls
  `resized_splits` with that slot as the start, not the window's rect: the
  window may still be sliding from the press before, and its rect would put
  the edge where it happens to be on the way. It returns the splits once
  the edge has moved, or none, which `resize_focused` sets with
  `Workspace::set_split_ratios`. Unit tests for every window, edge and
  direction in every layout of one to five windows, the bounds, the
  centered master, and down a stack every other edge kept where it is and
  the floor.
- `tiler/resize_actions.rs`: `resize_focused(plan, direction)`, dispatched
  from `actions.rs`.
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

- [ ] `Action::Resize(Direction)`, with `from_str`, `Display`, `scope()`
      and the CLI
- [ ] No default keys: the docs suggest `Alt+Ctrl+H/J/K/L` to add by hand,
      and say why they are not defaults and that a held key steps once
- [ ] `[layout] resize_step`, validated, in the template and with a hint
- [ ] `stepped_splits` in core, with unit tests
- [ ] The tiler actions, with tests for each direction, the opposite edge,
      nothing to move, monocle, the bounds, a burst and a key mid-drag
- [ ] Metrics scenarios for one press and a burst of nine
- [ ] Docs: `tiling-layout.md`, `keyboard-bindings.md`, `actions.md`,
      `configuration.md`, `cli.md`, and the website guide
- [ ] Manual test: resize every split of every layout from the keyboard, a
      burst, and a retile after them
- [ ] Update `.plans/plan.md`

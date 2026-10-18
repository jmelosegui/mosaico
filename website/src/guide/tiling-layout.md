# Tiling & Layouts

Mosaico automatically arranges windows on each monitor using a tiling
layout algorithm. Three layouts are available out of the box: **BSP**,
**VerticalStack**, and **ThreeColumn**.

## Layout Algorithms

### BSP (Binary Space Partitioning)

The default layout. The algorithm recursively divides the available
screen area:

1. **One window** -- fills the entire work area (minus gaps).
2. **Two or more windows** -- the area is split in half:
   - Even depth: **horizontal split** (side by side)
   - Odd depth: **vertical split** (stacked)
3. The first window gets the primary partition (controlled by the `ratio`
   setting).
4. Remaining windows are recursively laid out in the second partition.

This produces a balanced, grid-like arrangement where each new window gets
its own tile.

```
+-------+-------+
|       |   2   |
|   1   +---+---+
|       | 3 | 4 |
+-------+---+---+
```

### VerticalStack

A classic master/stack layout. The first window occupies a large pane on
the left (the "master"), and all remaining windows are stacked vertically
on the right.

```
+-----------+-----------+
|           |     2     |
|     1     +-----------+
| (master)  |     3     |
+-----------+-----------+
```

This layout works well when you have one main window you want to keep
visible (editor, browser) while referencing several smaller ones.

### ThreeColumn

A centered-master layout. The first window sits in the center column, and
additional windows alternate between stacks on the left and right sides.

```
+------+-----------+------+
|  2   |           |  3   |
+------+     1     +------+
|  4   | (master)  |  5   |
+------+-----------+------+
```

With only two windows it behaves like a simple left/right split. Once a
third window is added, the center master column appears and the side
stacks begin filling up.

## Cycling Layouts

Press **Alt + N** to cycle through layouts on the focused workspace:

> BSP -> VerticalStack -> ThreeColumn -> BSP -> ...

You can also run:

```
mosaico action cycle-layout
```

Or click the layout widget on the [status bar](status-bar.md#the-mouse) of
the monitor you want: a left click cycles its layout, a right click toggles
monocle there.

The layout change takes effect immediately and all windows are retiled.

## Configuration

Layout settings are in `config.toml`:

```toml
[layout]
gap = 8               # Pixel gap between windows (0-200)
ratio = 0.5           # Split ratio for the primary partition (0.1-0.9)
default = "bsp"       # Default layout for all workspaces
```

- **gap** -- the pixel spacing between windows and around the edges of the
  screen. Set to `0` for no gaps.
- **ratio** -- controls how much space the first window in each split
  receives. `0.5` means equal halves; `0.6` gives 60% to the primary
  window. A split you resize with the mouse keeps where you put it (see
  [Resizing Tiles](#resizing-tiles)); the others follow this ratio.
- **default** -- the layout algorithm used for any workspace that does not
  have an explicit override. Accepted values: `"bsp"`, `"vertical-stack"`,
  `"three-column"`.

### Per-Workspace Layout

You can assign a specific layout to individual workspaces using the
`[workspaces.layouts]` section. Workspace numbers range from 1 to 8.

The available layout values are:

| Value | Layout |
|-------|--------|
| `"bsp"` | Binary Space Partitioning (default) |
| `"vertical-stack"` | Master pane left, vertical stack right |
| `"three-column"` | Master pane center, stacks on both sides |

```toml
[layout]
default = "bsp"

[workspaces.layouts]
1 = "three-column"     # Workspace 1 always uses ThreeColumn
3 = "vertical-stack"   # Workspace 3 always uses VerticalStack
```

Workspaces without an entry in this table use the `layout.default`. You
can still cycle layouts at runtime with **Alt + N** -- the per-workspace
config only controls the initial layout when the daemon starts.

### Workspace Mode

`workspaces.mode` controls how a workspace switch propagates across
monitors:

```toml
[workspaces]
mode = "per-monitor"   # default
# mode = "global"
```

- **per-monitor** (default) -- a workspace switch only affects the focused
  monitor. Other monitors keep their current workspace, so you can flip
  one screen without disturbing the others.
- **global** -- a workspace switch flips every monitor to the same
  workspace number in lockstep, mirroring Windows virtual desktops. Each
  monitor still owns its own window list per workspace; the mode just
  syncs which workspace number every monitor is showing.

The mode is hot-reloaded: edit `config.toml` and the next workspace
switch follows the new mode without restarting the daemon.

## Monocle Mode

A per-monitor toggle that makes the focused window fill the entire work
area. Other windows remain in the workspace but are not repositioned.

- Toggle with `Alt + T` or `mosaico action toggle-monocle`
- The focus border color changes to the configured monocle color
- Each monitor has its own independent monocle state
- Dropping a window you drag onto another monitor in monocle ends monocle
  there, and the window joins its tiles (see
  [Dragging Windows](#dragging-windows))

## Dragging Windows

Drag a tiled window by its title bar and drop it on another tile to swap
the two. Drop it on another monitor to put it there, before the tile under
the pointer, or at the end when the pointer is over a gap or the status
bar. While you drag, a translucent preview in the focused border's color
shows where the window will land: on another monitor, that is the slot it
gets there, which in BSP is not always the tile under the pointer.

While you drag a window over another monitor, the windows there make
room for it, as they will once you drop it, gliding there when animations
are on, and the preview shows the slot they leave open. They move again
only when the window would go somewhere else, turning from wherever
they are, and they go back the same way when you leave that monitor,
press `Esc` or a hotkey, or use a status bar. When you drop it there,
they are already in place, or on their way, and only the window you
drop moves. A monitor in monocle is laid out with its windows side by
side while you drag over it, and goes back to monocle when you leave it;
dropping the window there ends monocle, and the window joins the others
side by side. Your own monitor keeps the window's slot until you drop
it, so you can drag back over it as it was.

The window goes back to its slot when you drop it on its own slot, in a
gap, on its monitor's status bar, or anywhere on its monitor in monocle,
and when Windows snaps or maximizes it on its monitor as you drop it.
Maximized by Windows on another monitor, it joins that monitor's windows
and stays maximized. Press `Esc` while dragging to leave everything as it
was, even with the pointer over another tile, and a click on the title
bar changes nothing either. Dragging a window by an edge resizes the
tiles instead (see [Resizing Tiles](#resizing-tiles)). A window Mosaico
does not tile can be dragged anywhere without moving the tiles under it.

The window you drag, by its title bar or an edge, takes focus as you
grab it, and keeps its border all along, which follows it as it moves
and changes size. While you drag, nothing else moves the window or the
pointer, whatever you press: a window that opens meanwhile is tiled
around it, and the pointer follows focus only once you let go. Pressing
a hotkey, or using a status bar, while dragging hides the preview, and
the window goes back to its slot when you drop it, or joins the end of
the other monitor's windows; pausing the hotkeys, from their key or the
**PAUSED** mark, leaves the drag as it is. Switching workspaces hides
it with the other windows there, and it stays there when you let go.

Set `drag_to_swap = false` under `[mouse]` in `config.toml` to have every
dropped window go back to its slot, or join the end of the other
monitor's windows, as before, with nothing making room for it while you
drag.

## Resizing Tiles

Drag the edge between two tiles to resize them: the windows on the other
side follow the edge as you drag it, at once, keeping the configured gap
from it, and when you let go, the split between them moves to where you
dropped the edge. Drag a corner where two splits meet to move both. Where
a split stops, the windows beyond it stop there too, and the window you
hold can overlap them until you let go, when it settles into its slot.

| Layout | Edges that resize |
|--------|-------------------|
| BSP | every edge between two tiles |
| VerticalStack | the edge between the master and the stack, and those between the windows of the stack |
| ThreeColumn | both edges of the master, the inner edge of the side columns, and the edges between the windows of a column; the master stays centered, so both its edges move together |

Dragging the edge between two windows of a stack, or of a side column,
trades height between those two only: every other window keeps its size.
The edges along the edges of the screen go back to where they were when
you let go, and a split stops a tenth of the way from either end; down a
stack, an edge stops where either of the two windows would have less than
a tenth of an equal share of the stack, which can be less than an app's
smallest height, so such a window can overlap the next. Press `Esc` while
resizing to leave everything as it was, the windows that followed the edge
going back to their tiles. A resize of 4 pixels or less on every edge
changes nothing either, as a click on the edge, and an edge you hold but
hardly move leaves its split where it was. While you hold a window by its
edge, nothing else moves it, as when you drag it by its title bar.
Pressing a hotkey, or using a status bar, while resizing ends the resize
without moving a split: the windows that followed the edge go back, and
the window goes back to its slot when you let go; pausing the hotkeys,
from their key or the **PAUSED** mark, leaves the resize as it is. With
"Show window contents while dragging" off in Windows, the windows follow
the edge only once you let go.

Each workspace keeps where you put its splits, and the heights you gave
its stacks, for each layout: cycling layouts brings them back, monocle
leaves them alone, and a monitor that disconnects and comes back brings
them back with its windows. When two windows swap, each takes the other's
tile at its size. A window that opens gets a split of its own at the
configured `ratio`, and in a stack you resized an equal share of it, which
the other windows there give up in proportion; a window that closes takes
its split with it and gives its height back to the others in proportion.
In three columns, a window that opens or closes before the last of the side
windows moves those after it to the other side column, and both columns go
back to equal heights. To put them all back at `ratio`, with equal heights
in the stacks, retile.

Set `resize_splits = false` under `[mouse]` in `config.toml` to have a
window you resize go back to its slot, as before.

## Retiling

Mosaico automatically retiles when windows are opened, closed, minimized,
restored, or moved between monitors. You can also manually retile at any
time:

- Press `Alt + Shift + R`
- Or run `mosaico action retile`

Retiling this way also puts every split you moved on the workspaces on
screen back at the configured `ratio`, and the stacks you resized back to
equal heights, in all their layouts. Workspaces out of view keep theirs,
and the retiles Mosaico does on its own keep them all.

## Window Management

Mosaico tiles a window if it meets all of the following criteria:

1. The window is visible
2. The window has a title bar (`WS_CAPTION` style)
3. The window is not a tool window
4. The window passes your [window rules](window-rules.md)

## Layout Changes Are Hot-Reloaded

Changes to `gap`, `ratio`, `default`, and `workspaces` in `config.toml`
are automatically applied while the daemon is running. All windows are
retiled immediately with the new settings.

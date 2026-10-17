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
  window.
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

## Dragging Windows

Drag a tiled window by its title bar and drop it on another tile to swap
the two. Drop it on another monitor to put it there, before the tile under
the pointer, or at the end when the pointer is over a gap or the status
bar. While you drag, a translucent preview in the focused border's color
shows where the window will land: on another monitor, that is the slot it
gets there, which in BSP is not always the tile under the pointer.

The window goes back to its slot when you drop it on its own slot, in a
gap, on its monitor's status bar, or anywhere on its monitor in monocle,
and when Windows snaps or maximizes it on its monitor as you drop it.
Maximized by Windows on another monitor, it joins that monitor's windows
and stays maximized. Press `Esc` while dragging to leave everything as it
was, even with the pointer over another tile, and a click on the title
bar changes nothing either. Dragging a window by an edge resizes it, and
it goes back to its slot when you let go. A window Mosaico does not tile
can be dragged anywhere without moving the tiles under it.

While you drag, nothing else moves the window or the pointer, whatever
you press: a window that opens meanwhile is tiled around it, and the
pointer follows focus only once you let go. Pressing a hotkey, or using a
status bar, while dragging hides the preview, and the window goes back to
its slot when you drop it, or joins the end of the other monitor's
windows; pausing the hotkeys, from their key or the **PAUSED** mark,
leaves the drag as it is. Switching workspaces hides it with the other
windows there, and it stays there when you let go.

Set `drag_to_swap = false` under `[mouse]` in `config.toml` to have every
dropped window go back to its slot, or join the end of the other
monitor's windows, as before.

## Retiling

Mosaico automatically retiles when windows are opened, closed, minimized,
restored, or moved between monitors. You can also manually retile at any
time:

- Press `Alt + Shift + R`
- Or run `mosaico action retile`

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

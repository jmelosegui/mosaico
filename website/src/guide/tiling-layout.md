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
`[layout.workspaces]` section. Workspace numbers range from 1 to 8.

```toml
[layout]
default = "bsp"

[layout.workspaces]
1 = "three-column"     # Workspace 1 always uses ThreeColumn
3 = "vertical-stack"   # Workspace 3 always uses VerticalStack
```

Workspaces without an entry in this table use the `default` layout. You
can still cycle layouts at runtime with **Alt + N** -- the per-workspace
config only controls the initial layout when the daemon starts.

## Monocle Mode

A per-monitor toggle that makes the focused window fill the entire work
area. Other windows remain in the workspace but are not repositioned.

- Toggle with `Alt + T` or `mosaico action toggle-monocle`
- The focus border color changes to the configured monocle color
- Each monitor has its own independent monocle state

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

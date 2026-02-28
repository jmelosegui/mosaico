# Tiling & Layouts

Mosaico uses a Binary Space Partitioning (BSP) algorithm to automatically
arrange windows on each monitor.

## How BSP Tiling Works

The algorithm recursively divides the available screen area:

1. **One window** -- fills the entire work area (minus gaps).
2. **Two or more windows** -- the area is split in half:
   - Even depth: **horizontal split** (side by side)
   - Odd depth: **vertical split** (stacked)
3. The first window gets the primary partition (controlled by the `ratio`
   setting).
4. Remaining windows are recursively laid out in the second partition.

This produces a balanced, grid-like arrangement where each new window gets
its own tile.

## Configuration

Layout settings are in `config.toml`:

```toml
[layout]
gap = 8       # Pixel gap between windows (0-200)
ratio = 0.5   # Split ratio for the primary partition (0.1-0.9)
```

- **gap** -- the pixel spacing between windows and around the edges of the
  screen. Set to `0` for no gaps.
- **ratio** -- controls how much space the first window in each split
  receives. `0.5` means equal halves; `0.6` gives 60% to the primary
  window.

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

Changes to `gap` and `ratio` in `config.toml` are automatically applied
while the daemon is running. All windows are retiled immediately with the
new settings.

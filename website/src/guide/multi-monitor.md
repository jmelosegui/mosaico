# Multi-Monitor

Mosaico manages each monitor independently with its own workspaces, layout,
and monocle state. Windows can be navigated and moved across monitors
seamlessly.

By default, switching workspaces only flips the focused monitor (per-monitor
mode). Set `workspaces.mode = "global"` in `config.toml` to make every
monitor switch to the same workspace number in lockstep, like Windows
virtual desktops. See [Workspaces -> Workspace Mode](workspaces.md#workspace-mode).

## Monitor Detection

Monitors are automatically enumerated and sorted left-to-right by their
x-coordinate. Each monitor's work area excludes the Windows taskbar and the
Mosaico status bar (if enabled).

## Unplugging and Reconnecting Monitors

Mosaico remembers your layout when a monitor goes away, whether you unplug
a cable, undock a laptop, or switch a KVM to another computer:

- The windows of the disconnected monitor move to the **same workspace
  number** on the primary monitor. Windows from workspace 3 are on
  workspace 3 while the monitor is gone.
- When the same monitor is connected again, its windows go back to the
  workspace and position they had, and the monitor gets back its active
  workspace, layouts, and monocle state.
- A window you move yourself while the monitor is gone (to another
  workspace or another monitor) stays where you put it.
- A different monitor that happens to sit in the same place, such as one
  at the office, does not take over the workspaces of your home monitor.
  It starts with its own empty workspaces.

Monitors are recognised by the identity they report (EDID make, model and
serial, and the port they are plugged into), so this works even though
Windows renumbers every monitor on each change. Two identical monitors
are told apart by serial number or by port. `mosaico debug list` shows
the identity of each connected monitor.

The layout is kept in memory, so it survives sleep but not a restart of
the Mosaico daemon.

## Cross-Monitor Navigation

Focus navigation with `Alt + H/L` (left/right) can cross monitor
boundaries. When there is no window in the requested direction on the
current monitor, focus moves to the nearest window on the adjacent monitor.

Up/Down navigation (`Alt + J/K`) stays within the current monitor.

## Moving Windows Across Monitors

Move actions with `Alt + Shift + H/L` transfer windows between monitors:

- **Moving right** -- the window is placed at the leftmost position in the
  target monitor's BSP layout.
- **Moving left** -- the window is placed at the rightmost position.

Both the source and target monitors are retiled after the move.

Up/Down move actions (`Alt + Shift + J/K`) swap windows within the same
monitor.

## Manual Dragging

If you drag a tiled window by its title bar to a different monitor and drop
it there, it joins that monitor's windows before the tile under the
pointer, or at the end when the pointer is over a gap or its status bar,
and that monitor takes focus. A preview shows where it will land while you
drag, and the windows of that monitor make room for it as you drag over
them, going back when you leave; one in monocle is laid out side by side
meanwhile, and leaves monocle when you drop the window there. Where the
pointer is decides, not where most of the window is, so a window Windows
rescales on its way to a monitor of another scale lands where you point.
Both monitors are retiled. A window Windows maximizes there as you drop it
joins that monitor too, and stays maximized. See
[Dragging Windows](tiling-layout.md#dragging-windows).

With `drag_to_swap = false` under `[mouse]` in `config.toml`, or for a
window Mosaico starts tiling as you drop it, the window joins the end of
the windows of the monitor it is on.

## DPI Awareness

Mosaico uses per-monitor DPI awareness (V2) to ensure accurate pixel
positioning on mixed-DPI multi-monitor setups. This is handled automatically
and requires no configuration.

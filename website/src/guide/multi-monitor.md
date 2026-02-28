# Multi-Monitor

Mosaico manages each monitor independently with its own workspaces, layout,
and monocle state. Windows can be navigated and moved across monitors
seamlessly.

## Monitor Detection

Monitors are automatically enumerated and sorted left-to-right by their
x-coordinate. Each monitor's work area excludes the Windows taskbar and the
Mosaico status bar (if enabled).

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

If you manually drag a window to a different monitor, Mosaico detects the
monitor change and reassigns the window automatically. Both monitors are
retiled.

## DPI Awareness

Mosaico uses per-monitor DPI awareness (V2) to ensure accurate pixel
positioning on mixed-DPI multi-monitor setups. This is handled automatically
and requires no configuration.

# Status Bar

Mosaico displays a configurable status bar at the top of each monitor. The
bar shows workspace indicators, layout mode, and system information using
per-pixel alpha rendering for a polished, translucent appearance.

## Architecture

What a bar shows is decided in `mosaico-core`, so every platform shows the
same bar. The platform crate only reads the machine and draws.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig`, `BarColors`, `WidgetConfig` |
| `crates/mosaico-core/src/bar/manager.rs` | `BarManager` -- which monitors get a bar, and refreshing them |
| `crates/mosaico-core/src/bar/content.rs` | `BarContent`, `BarItem` -- what each bar shows, in drawing order |
| `crates/mosaico-core/src/bar/widget_text.rs` | Each widget's text, and when it shows |
| `crates/mosaico-core/src/bar/state.rs` | `BarState` -- what a monitor's bar shows of the tiler and daemon |
| `crates/mosaico-core/src/bar/readings.rs` | `SystemReadings` -- CPU, memory, media and time for one refresh |
| `crates/mosaico-core/src/bar/system_info.rs` | `SystemInfo` -- the machine readings a platform reports |
| `crates/mosaico-core/src/bar/renderer.rs` | `BarRenderer`, `BarSurface` -- what a platform implements to draw bars |
| `crates/mosaico-core/src/bar/cpu.rs` | `CpuTracker` -- CPU usage from processor time counters |
| `crates/mosaico-core/src/bar/time.rs` | `LocalTime` and the clock's format strings |
| `crates/mosaico-core/src/tiler/bar_states.rs` | `Tiler::bar_states()` -- one `BarState` per monitor |
| `crates/mosaico-windows/src/bar/mod.rs` | `Win32BarRenderer` |
| `crates/mosaico-windows/src/bar/window.rs` | `Bar` -- per-monitor overlay window, a `BarSurface` |
| `crates/mosaico-windows/src/bar/render.rs` | DIB rendering pipeline |
| `crates/mosaico-windows/src/bar/text.rs` | `DrawCtx`, text and pill rendering helpers |
| `crates/mosaico-windows/src/bar/widgets/*.rs` | Draws the items: pills, separators, workspaces, window icon |
| `crates/mosaico-windows/src/system_info.rs` | `Win32SystemInfo` -- CPU, memory and time from Win32 |

### Key Types

- `BarManager` -- decides which monitors get a bar, the space it reserves
  and what each one shows; owns the config, the resolved colors and the
  CPU tracker, and draws through a `BarRenderer`
- `BarContent` / `BarItem` -- the items one bar draws on each side: the
  workspace numbers, the window icon, pills with a label and colors, and
  separators
- `BarState` -- what one monitor's bar shows of the tiler and the daemon
- `SystemReadings` -- CPU, memory, media and time, the same for every bar
- `SystemInfo` -- trait for the raw machine readings a platform reports
- `BarRenderer` / `BarSurface` -- traits to make, draw, hide and drop a bar
- `BarConfig` -- full bar configuration from `bar.toml`
- `BarColors` -- 9 color fields for all bar UI elements
- `WidgetConfig` -- tagged enum for widget type and settings
- `Bar` -- a single monitor's status bar on Windows; wraps a layered Win32
  `HWND`
- `DrawCtx` -- GDI drawing context bundling DC, pixel buffer, and dimensions

## Platform Contracts

A platform gives the bar two things:

- `SystemInfo` reports raw readings: processor time counters, the share of
  memory in use, what is playing and the local time. Each call must return
  at once, since the daemon makes them on its own thread every second.
  `CpuTracker` turns two counter readings into a percentage.
- `BarRenderer::create` makes a hidden bar at the top of a monitor's work
  area. The `BarSurface` it returns draws a `BarContent` in the config's
  style and shows it, hides, and is removed when dropped. A bar never takes
  focus or pointer input, stays out of the taskbar and task switcher, and is
  never reported to the tiler as a window.

The full contracts are on the traits in `crates/mosaico-core/src/bar/`.

## Window Properties

Each bar is a Win32 window with:

- `WS_POPUP` -- no frame
- `WS_EX_LAYERED` -- per-pixel alpha transparency
- `WS_EX_TOOLWINDOW` -- hidden from taskbar and Alt+Tab
- `WS_EX_TRANSPARENT` -- click-through

Each draw moves it to the top of the z-order (`HWND_TOP`) without
activating it (`SWP_NOACTIVATE`).

Window class `MosaicoBar` is registered once. `WINEVENT_SKIPOWNPROCESS`
prevents bar windows from triggering tiling events. The window procedure
ignores `WM_CLOSE`, so End task cannot destroy a bar that `BarManager`
still holds.

## Rendering Pipeline

`render_bar()` executes the full rendering pass:

1. Creates a compatible DC and 32-bit BGRA DIB section
2. Fills background with configurable opacity (premultiplied alpha)
3. Creates a GDI font from the configured font name and size
4. Builds a `DrawCtx` with the DC, pixel buffer, and dimensions
5. Calls `widgets::draw_left()` with the content's left items
6. Calls `widgets::draw_right()` with the content's right items
7. Calls `UpdateLayeredWindow` with `ULW_ALPHA` to atomically present

A step that fails is reported to `BarManager`, which logs it at debug level
and draws again on the next refresh.

### Text Rendering

GDI `TextOutW` renders text onto the DIB. Since GDI always writes zero
alpha for text pixels, `fix_alpha_region()` post-processes the affected
region to restore full opacity based on color difference from the
background.

### Pill Rendering

Widgets are drawn inside rounded-rectangle "pills" with configurable
radius (`pill_radius`) and border width (`pill_border_width`, 0 for none).
`draw_pill()` renders directly into the pixel buffer, testing each pixel
against the outer and the inset rounded rectangle (`in_rounded_rect`).

## Widget System

### Available Widgets

| Type | Description | Extra Config |
|------|-------------|--------------|
| `workspaces` | Numbered workspace indicators (1-8), active highlighted | -- |
| `active_window` | Icon of the focused window (bitmap, not text) | -- |
| `layout` | Current layout name ("BSP") and monocle indicator ("M") | -- |
| `clock` | Current time | `format` (strftime, default `"%H:%M:%S"`) |
| `date` | Current date | `format` (strftime, default `"%A %d %B %Y"`) |
| `ram` | System RAM usage percentage (`GlobalMemoryStatusEx` on Windows) | -- |
| `cpu` | CPU usage percentage between refreshes (`GetSystemTimes` on Windows) | -- |
| `update` | Update notification when newer version is available | -- |
| `media` | Currently playing track (not read on Windows yet, see below) | `max_length` (default 40) |
| `paused` | Indicator shown when mosaico hotkeys are paused | `color` (default `"red"`) |

Each widget can be independently enabled/disabled and assigned a custom icon.

The `paused` widget is auto-hidden when hotkeys are not paused. It is
included in the default bar config and added automatically to existing
`bar.toml` files via the widget auto-merge mechanism (see below).

### What Each Bar Shows

`BarContent::build()` turns the config, the monitor's `BarState` and the
`SystemReadings` into the items to draw:

- A widget shows unless it is disabled, or it is the `update` widget with
  no update, the `media` widget with nothing playing, or the `paused`
  widget while hotkeys work.
- A pill's label is the widget's icon, a space, and its text; either may
  be missing.
- When `separator` is set, it goes between every two widgets that show,
  with a space on each side.
- A pill widget's `color` tints its text and outline. Without one, the
  pill uses the `foreground` and `pill_border` colors. On the left, the
  workspace container and the window icon keep the bar's colors.
- The workspace numbers and the window icon are only drawn on the left
  side. On the right, those widgets are plain pills showing their icon.
- The `active_window` widget keeps its place on monitors without a focused
  window, so the separators around it still show; it just takes no space.

### Active Window Icon

The `active_window` widget extracts the application icon from the focused
window's process executable via `SHGetFileInfoW` and renders it as a BGRA
bitmap inside a square pill. The icon is drawn at its native size, capped
at about 60% of the pill height, and centered.

When no window is focused or icon extraction fails, the widget collapses
to zero width. Icons are extracted fresh on each render (no caching) since
`SHGetFileInfoW` is fast and the focused window can change at any time.

### Media

The `media` widget shows what is playing as "Artist - Title", or only the
title when there is no artist. Long titles are truncated to `max_length`
characters (default 40) with an ellipsis. When nothing is playing, or the
platform cannot tell, the widget is invisible (same pattern as the Update
widget).

Windows does not read the media session yet. The
`GlobalSystemMediaTransportControlsSessionManager` (GSMTC) query blocks
forever on the daemon thread, so `Win32SystemInfo` reports that it cannot
tell, and the widget stays hidden until a non-blocking query is added.

### Widget Placement

- Left widgets are rendered left-to-right with separators
- Right widgets are rendered right-to-left
- Widgets are configured as `[[left]]` and `[[right]]` arrays in `bar.toml`

### BarState

`BarState` holds what one monitor's bar shows of the tiler and the daemon:

- `active_workspace` -- 0-indexed active workspace on this monitor
- `workspace_count` -- number of workspaces on the monitor
- `layout` -- layout of the active workspace, shown as "BSP", "VStack" or
  "3Col"
- `monocle` -- whether monocle mode is active
- `focused_window` -- `WindowId` of the focused window (`None` on non-focused monitors)
- `update_text` -- update notification string (empty if up to date)
- `paused` -- whether mosaico hotkeys are currently paused

`Tiler::bar_states()` builds one per monitor on each render cycle, and
`TilingManager::bar_states()` passes it the update text and pause state.

The system widgets read `SystemReadings` instead, which `BarManager` takes
once per refresh for all bars, and not at all while no monitor has a bar:
CPU usage (only while a `cpu` widget is configured), memory in use (only
with an enabled `ram` widget), what is playing (only with an enabled
`media` widget) and the local time (only with an enabled `clock` or `date`
widget). A reading that fails shows as 0, or as nothing playing.

## Widget Auto-Merge

On daemon startup, `ConfigStore::merge_missing_bar_widgets()` compares the
widget types present in the user's `bar.toml` against the built-in defaults.
Any widget type not already present is appended to the file automatically, so
new widgets added in future versions are picked up without requiring users to
edit their config. The comparison uses `std::mem::discriminant` (enum variant
identity) on each side, `[[left]]` and `[[right]]`, so a widget the user
configured differently is not added twice. A default widget type removed from
its side is added back on the next start; to hide a widget, set
`enabled = false` on it instead.

## Bar Manager

`BarManager` orchestrates bar lifecycle across monitors:

- `new(renderer, system, config, monitor_rects, theme)` -- creates one bar
  per selected monitor through the renderer, resolves colors from theme,
  and starts a `CpuTracker` when a `cpu` widget is configured
- `update(states)` -- reads the system once, then builds each bar's content
  from its monitor's state and draws it; does nothing while no monitor has
  a bar
- `bar_height()` -- returns the bar height while at least one monitor has a
  bar, 0 otherwise, so a bar that no monitor gets reserves no space (the
  tiler reads an empty monitor list as every monitor)
- `bar_monitor_indices()` -- returns which monitor indices have bars
- `resolve_colors(theme)` -- re-resolves every color on theme change, the
  `[colors]` table and each widget's `color`
- `rebuild_for_monitors(monitor_rects, theme)` -- replaces the bars after a
  display change
- `reload(config)` -- recreates bars with new config, returns new height
- `hide_all()` -- hides all bars on shutdown

A monitor whose bar cannot be made goes without one, and the failure is
logged. The manager keeps the config as loaded (`raw_config`) apart from
the resolved one, so a theme change resolves every color again from what
`bar.toml` says: a color given by name follows the new theme, and an empty
one takes the new theme's default. The daemon creates the manager with
`Win32BarRenderer` and `Win32SystemInfo`.

## Work Area Reservation

The daemon subtracts the bar height from each monitor's work area before
passing it to the `TilingManager`. This ensures BSP layout computation
accounts for the bar space.

- `adjust_work_areas_for_bar(bar_height, bar_monitors)` -- offsets
  `work_area.y` and `work_area.height` for selected monitors
- `reset_and_adjust_work_areas(bar_height, bar_monitors)` -- resets to
  OS values first to prevent offset accumulation on config reload

## Refresh Strategy

- **Tiling state changes**: bars re-render on every window event, action,
  and workspace switch
- **System widgets**: a 1-second `DaemonMsg::Tick` triggers clock, RAM,
  CPU, and date refresh
- **Config reload**: `ConfigReload::Bar` recreates bars, resets work areas

## Per-Monitor Filtering

The `monitors` field in `bar.toml` controls which monitors display bars.
An empty list (default) shows bars on all monitors. Specifying monitor
indices (0-based) restricts bars to those monitors only.

## Configuration

See [configuration.md](configuration.md) for the full `bar.toml` format.

## Design Decisions

- **Content in core, drawing in the platform**: which widgets show, their
  text, order, separators and colors are decided in `mosaico-core` and unit
  tested against a fake renderer and machine, so another platform only
  implements `SystemInfo` and `BarRenderer`.
- **`UpdateLayeredWindow`** provides flicker-free atomic rendering without
  needing a `WM_PAINT` handler.
- **DIB section with 32-bit BGRA** gives full per-pixel alpha control,
  enabling translucent backgrounds and anti-aliased pills.
- **GDI for text** is used because it is the simplest Win32 text API that
  works with DIB sections. The alpha fix-up is a known workaround.
- **`CpuTracker`** computes usage from the change in processor time
  counters between two refreshes rather than from performance counters,
  keeping the platform side a plain read. It only exists while the CPU
  widget is configured.
- **One reading per refresh**: `SystemReadings` is taken once for all bars,
  so every monitor shows the same CPU, memory and time.
- **One `Bar` per monitor** rather than a single shared bar, because each
  monitor has independent workspace state and work area.
- **The raw config stored separately** from the resolved one lets a theme
  change re-resolve every color, widget colors included, without
  re-reading `bar.toml`.

# Status Bar

Mosaico displays a configurable status bar at the top of each monitor. The
bar shows workspace indicators, layout mode, and system information using
per-pixel alpha rendering for a polished, translucent appearance.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig`, `BarColors`, `WidgetConfig` |
| `crates/mosaico-windows/src/bar/mod.rs` | `Bar` -- per-monitor overlay window |
| `crates/mosaico-windows/src/bar/render.rs` | DIB rendering pipeline |
| `crates/mosaico-windows/src/bar/text.rs` | `DrawCtx`, text and pill rendering helpers |
| `crates/mosaico-windows/src/bar/widgets/mod.rs` | Widget system, `BarState`, draw orchestration |
| `crates/mosaico-windows/src/bar/widgets/*.rs` | Individual widget implementations |
| `crates/mosaico-windows/src/bar_manager.rs` | `BarManager` -- multi-monitor bar lifecycle |

### Key Types

- `Bar` -- a single monitor's status bar; wraps a layered Win32 `HWND`
- `BarManager` -- manages `Bar` instances across monitors, owns config and
  CPU tracker
- `BarConfig` -- full bar configuration from `bar.toml`
- `BarColors` -- 9 color fields for all bar UI elements
- `WidgetConfig` -- tagged enum for widget type and settings
- `BarState` -- snapshot of tiling and system state for rendering
- `DrawCtx` -- GDI drawing context bundling DC, pixel buffer, and dimensions

## Window Properties

Each bar is a Win32 window with:

- `WS_POPUP` -- no frame
- `WS_EX_LAYERED` -- per-pixel alpha transparency
- `WS_EX_TOOLWINDOW` -- hidden from taskbar and Alt+Tab
- `WS_EX_TOPMOST` -- always on top
- `WS_EX_TRANSPARENT` -- click-through

Window class `MosaicoBar` is registered once. `WINEVENT_SKIPOWNPROCESS`
prevents bar windows from triggering tiling events.

## Rendering Pipeline

`render_bar()` executes the full rendering pass:

1. Creates a compatible DC and 32-bit BGRA DIB section
2. Fills background with configurable opacity (premultiplied alpha)
3. Creates a GDI font from the configured font name and size
4. Builds a `DrawCtx` with the DC, pixel buffer, and dimensions
5. Calls `widgets::draw_left()` for left-aligned widgets
6. Calls `widgets::draw_right()` for right-aligned widgets
7. Calls `UpdateLayeredWindow` with `ULW_ALPHA` to atomically present

### Text Rendering

GDI `TextOutW` renders text onto the DIB. Since GDI always writes zero
alpha for text pixels, `fix_alpha_region()` post-processes the affected
region to restore full opacity based on color difference from the
background.

### Pill Rendering

Widgets are drawn inside rounded-rectangle "pills" with configurable
radius and optional 1px borders. `draw_pill()` renders directly into the
pixel buffer using geometry helpers (`in_rounded_rect`, `is_border_pixel`).

## Widget System

### Available Widgets

| Type | Description | Extra Config |
|------|-------------|--------------|
| `workspaces` | Numbered workspace indicators (1-8), active highlighted | -- |
| `layout` | Current layout name ("BSP") and monocle indicator ("M") | -- |
| `clock` | Current time | `format` (strftime, default `"%H:%M:%S"`) |
| `date` | Current date | `format` (strftime, default `"%A %d %B %Y"`) |
| `ram` | System RAM usage percentage via `GlobalMemoryStatusEx` | -- |
| `cpu` | CPU usage percentage via `GetSystemTimes` delta tracking | -- |
| `update` | Update notification when newer version is available | -- |

Each widget can be independently enabled/disabled and assigned a custom icon.

### Widget Placement

- Left widgets are rendered left-to-right with separators
- Right widgets are rendered right-to-left
- Widgets are configured as `[[left]]` and `[[right]]` arrays in `bar.toml`

### BarState

The `BarState` struct provides the data each widget needs:

- `active_workspace` -- 0-indexed active workspace on this monitor
- `workspace_count` -- total workspaces with windows
- `layout_name` -- "BSP"
- `monocle` -- whether monocle mode is active
- `cpu_usage` -- current CPU percentage (from `CpuTracker`)
- `update_text` -- update notification string (empty if up to date)

`TilingManager::bar_states()` produces a `Vec<BarState>` snapshot for all
monitors on each render cycle.

## Bar Manager

`BarManager` orchestrates bar lifecycle across monitors:

- `new(config, monitor_rects, theme)` -- creates one `Bar` per selected
  monitor, resolves colors from theme, optionally allocates `CpuTracker`
- `update(states)` -- renders all bars with current state, samples CPU
- `bar_height()` -- returns bar height if enabled, 0 otherwise
- `bar_monitor_indices()` -- returns which monitor indices have bars
- `resolve_colors(theme)` -- re-resolves colors on theme change
- `reload(config)` -- recreates bars with new config, returns new height
- `hide_all()` -- hides all bars on shutdown

The manager stores `raw_colors` (pre-resolution) separately from resolved
colors to support re-resolution when the theme changes.

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

- **`UpdateLayeredWindow`** provides flicker-free atomic rendering without
  needing a `WM_PAINT` handler.
- **DIB section with 32-bit BGRA** gives full per-pixel alpha control,
  enabling translucent backgrounds and anti-aliased pills.
- **GDI for text** is used because it is the simplest Win32 text API that
  works with DIB sections. The alpha fix-up is a known workaround.
- **`CpuTracker`** uses `GetSystemTimes` delta between ticks rather than
  performance counters, keeping the implementation simple and dependency-free.
  It is lazily allocated only when the CPU widget is configured.
- **One `Bar` per monitor** rather than a single shared bar, because each
  monitor has independent workspace state and work area.
- **Raw colors stored separately** from resolved colors enables re-resolution
  when the theme changes without re-reading `bar.toml`.

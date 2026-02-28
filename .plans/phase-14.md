# Phase 14: Status Bar

**Status:** Complete

**Goal:** Add a configurable status bar to the top of each monitor displaying
workspace indicators, layout mode, and system information.

## Overview

The bar is a thin Win32 overlay window anchored to the top of each monitor.
It reserves space from the work area so tiled windows do not overlap it.
The bar renders using `UpdateLayeredWindow` (same technique as the focus
border) for flicker-free per-pixel alpha compositing.

## Reference Design

```
[1] 2 3 4 5 6 7 | BSP | M     Update available! v0.1.40 | RAM: 24% | Sunday 22 February 2026 | 02:15:35
^^^^^^^^^^^^^^^^^  ^^^   ^     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
workspace numbers  layout monocle                              right-aligned system widgets
```

- Left section: workspace indicators (active one highlighted), layout name,
  monocle indicator
- Right section: update notification, RAM usage, date, time

## Architecture

### New Files

```
crates/
  mosaico-core/
    src/
      config/
        bar.rs              # BarConfig, BarColors, WidgetConfig, defaults
      config/
        template.rs         # Updated: generate_bar() for bar.toml template
        loader.rs           # Updated: bar_path(), load_bar(), try_load_bar()
  mosaico-windows/
    src/
      bar/
        mod.rs              # Bar struct, window creation, lifecycle
        render.rs           # DIB rendering, text layout, widget painting
        widgets.rs          # Widget data providers (clock, RAM, date, etc.)
      config_watcher.rs     # Updated: poll bar.toml alongside other files
```

### Key Types

- `BarConfig` — top-level bar config: `enabled`, `height`, `font_size`,
  `padding`, `colors`, `left` (widgets), `right` (widgets)
- `BarColors` — `background`, `foreground`, `active_workspace`,
  `inactive_workspace`, `separator`, `accent`
- `WidgetConfig` — tagged enum with `type` field: `Workspaces`, `Layout`,
  `Clock { format }`, `Date { format }`, `Ram`, `Update`
- `Bar` — wraps a layered HWND per monitor; methods: `new()`, `show()`,
  `update()`, `hide()`

### Integration Points

- **Work area adjustment**: the daemon must subtract the bar height from each
  monitor's work area before passing it to the tiling manager. This ensures
  BSP layout computation accounts for the bar.
- **Workspace state**: the bar needs to know the active workspace index and
  total workspace count per monitor, sourced from `TilingManager`.
- **Layout state**: the bar displays the current layout name ("BSP") and
  monocle status.
- **Periodic refresh**: system widgets (clock, RAM) need a timer-based
  refresh, likely a 1-second tick via `SetTimer` or a dedicated thread.
- **Config hot-reload**: the config watcher polls `bar.toml` alongside the
  other config files. Changes are validated via `try_load_bar()` before
  applying. A new `ConfigReload::Bar(BarConfig)` variant carries validated
  reloads to the daemon.

## Configuration

The bar has its own dedicated configuration file, `~/.config/mosaico/bar.toml`,
separate from the main `config.toml`. This keeps bar-specific settings
(colors, widgets, layout) independently manageable and avoids bloating the
main config.

### `bar.toml`

```toml
# Bar appearance
enabled = true
height = 28
font_size = 13
padding = 8               # Horizontal padding inside the bar

# Colors
[colors]
background = "#1e1e2e"
foreground = "#cdd6f4"
active_workspace = "#89b4fa"
inactive_workspace = "#585b70"
separator = "#45475a"
accent = "#f38ba8"         # Used for update notifications, alerts

# Left-side widgets (rendered left-to-right)
[[left]]
type = "workspaces"

[[left]]
type = "layout"

# Right-side widgets (rendered right-to-left)
[[right]]
type = "clock"
format = "%H:%M:%S"       # strftime-style format

[[right]]
type = "date"
format = "%A %d %B %Y"    # e.g., "Sunday 22 February 2026"

[[right]]
type = "ram"

[[right]]
type = "update"
```

### Widget Configuration

Each widget is a TOML table with a `type` field and optional widget-specific
settings. Available widget types:

| Type | Description | Extra Fields |
|------|-------------|--------------|
| `workspaces` | Numbered workspace indicators with active highlight | — |
| `layout` | Current layout name (BSP) and monocle indicator | — |
| `clock` | Current time | `format` (strftime, default `"%H:%M:%S"`) |
| `date` | Current date | `format` (strftime, default `"%A %d %B %Y"`) |
| `ram` | System RAM usage percentage | — |
| `update` | Update availability notification | — |

### Loading

- `bar_path()` returns `~/.config/mosaico/bar.toml`
- `load_bar()` loads with silent fallback to defaults (bar enabled with
  default widgets)
- `try_load_bar()` returns `Result<BarConfig, String>` for `doctor` and
  hot-reload validation
- `mosaico init` generates a fully-commented `bar.toml` template
- The config watcher polls `bar.toml` alongside `config.toml` and `rules.toml`

## Window Properties

- `WS_POPUP` — no frame
- `WS_EX_LAYERED` — per-pixel alpha
- `WS_EX_TOOLWINDOW` — hidden from taskbar and Alt+Tab
- `WS_EX_TOPMOST` — always on top
- `WS_EX_TRANSPARENT` — click-through (initial version; clickable workspaces
  can be added later)
- `WINEVENT_SKIPOWNPROCESS` already prevents bar windows from triggering
  tiling events

## Rendering

Use `UpdateLayeredWindow` with a DIB section (same approach as the focus
border). For text rendering, use GDI `SelectObject`/`TextOutW` onto the DIB's
HDC with a `HFONT` created from the configured font size.

### Render Pipeline

1. Create DIB section matching bar dimensions
2. Fill background color
3. Render left widgets (workspaces, layout) left-to-right
4. Render right widgets (clock, date, RAM) right-to-left
5. Call `UpdateLayeredWindow` to apply

### Widget Rendering

Each widget is a self-contained unit that:
- Computes its text content (e.g., "RAM: 24%", "02:15:35")
- Measures its width via `GetTextExtentPoint32W`
- Draws itself at the assigned position

## Work Area Reservation

Two approaches to reserve space:

**Option A — Manual offset**: subtract bar height from each monitor's
`work_area.y` and `work_area.height` in the daemon before passing to
`TilingManager`. Simple, no OS-level side effects.

**Option B — AppBar registration**: use `SHAppBarMessage(ABM_NEW)` to
register the bar as an application bar. The OS automatically adjusts work
areas for all applications. More correct, but adds Win32 API complexity.

Recommendation: start with Option A for simplicity.

## Refresh Strategy

- **Workspace/layout changes**: re-render immediately when the tiling manager
  state changes (workspace add/remove, focus change, layout toggle)
- **System widgets**: use a 1-second timer to refresh clock, RAM, and date
- **Config reload**: re-render on `ConfigReload::Bar` with new colors/size/
  widgets

## Tasks

- [x] Define `BarConfig`, `BarColors`, `WidgetConfig` in `mosaico-core` with defaults and serde
- [x] Add `bar_path()`, `load_bar()`, `try_load_bar()` to config loader
- [x] Add `generate_bar()` to template generator
- [x] Update `mosaico init` to write `bar.toml`
- [x] Create bar window with layered overlay properties (one per monitor)
- [x] Implement DIB-based text rendering pipeline
- [x] Implement workspace indicator widget (numbers, active highlight)
- [x] Implement layout indicator widget (BSP/Monocle)
- [x] Implement clock widget with configurable format
- [x] Implement date widget with configurable format
- [x] Implement RAM usage widget
- [x] Implement update notification widget
- [x] Adjust work area to account for bar height
- [x] Wire bar updates to tiling manager state changes
- [x] Add 1-second refresh timer for system widgets
- [x] Add `ConfigReload::Bar` variant and poll `bar.toml` in config watcher
- [x] Add bar validation to `mosaico doctor`
- [x] Build, lint, test
- [x] Update documentation

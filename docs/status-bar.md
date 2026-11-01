# Status Bar

Mosaico displays a configurable status bar at the top of each monitor. The
bar shows workspace indicators, layout mode, and system information using
per-pixel alpha rendering for a polished, translucent appearance. A click on
a workspace number goes to that workspace and the wheel over the bar steps
through the workspaces, on the bar's own monitor, and a click on the layout
or paused widget, or on any widget a binding names, runs an action (see
[Pointer Input](#pointer-input)).

## Architecture

What a bar shows, and what the pointer does on it, is decided in
`mosaico-core`, so every platform shows the same bar and does the same with
a click. The platform crate only reads the machine, draws, and reports what
the pointer does.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/config/bar.rs` | `BarConfig`, `BarColors`, `WidgetConfig` |
| `crates/mosaico-core/src/config/bar_mouse.rs` | `BarMouseConfig`, `BarScroll`, `ClickBinding` -- the `[mouse]` section and its `[[mouse.click]]` entries |
| `crates/mosaico-core/src/config/widget_kind.rs` | `WidgetKind` -- a widget's type, named as `type` names it |
| `crates/mosaico-core/src/bar/clicks.rs` | `WidgetClicks` -- what a click on each kind of widget runs; how each entry is checked, and the entries it leaves out, `RejectedClick` and `ClickProblem` (`clicks_rejected.rs`) |
| `crates/mosaico-core/src/bar/manager.rs` | `BarManager` -- which monitors get a bar, and refreshing them; making them (`manager_bars.rs`), what it keeps of each (`manager_slot.rs`) and the pointer on them (`manager_pointer.rs`) |
| `crates/mosaico-core/src/bar/content.rs` | `BarContent`, `BarItem` -- what each bar shows, in drawing order; the items of a side (`content_items.rs`) and what each is to the pointer (`content_regions.rs`) |
| `crates/mosaico-core/src/bar/pointer.rs` | `BarId`, `Button`, `PointerEvent`, `PointerSink` -- what a platform reports, and where |
| `crates/mosaico-core/src/bar/region.rs` | `ItemRegion`, `BarTarget`, `WidgetRef`, `BarSide`, `hit()` -- where a bar drew its items, and which one the pointer is over |
| `crates/mosaico-core/src/bar/press.rs` | `Press` -- a press and a release on the same item is a click |
| `crates/mosaico-core/src/bar/wheel.rs` | `WheelSteps`, `WHEEL_QUIET` -- wheel turns added up until the wheel goes quiet |
| `crates/mosaico-core/src/bar/widget_text.rs` | Each widget's text, and when it shows |
| `crates/mosaico-core/src/bar/state.rs` | `BarState` -- what a monitor's bar shows of the tiler and daemon |
| `crates/mosaico-core/src/bar/readings.rs` | `SystemReadings` -- CPU, memory, media and time for one refresh |
| `crates/mosaico-core/src/bar/system_info.rs` | `SystemInfo` -- the machine readings a platform reports |
| `crates/mosaico-core/src/bar/renderer.rs` | `BarRenderer`, `BarSurface` -- what a platform implements to draw bars |
| `crates/mosaico-core/src/bar/cpu.rs` | `CpuTracker` -- CPU usage from processor time counters |
| `crates/mosaico-core/src/bar/time.rs` | `LocalTime` and the clock's format strings |
| `crates/mosaico-core/src/tiler/bar_states.rs` | `Tiler::bar_states()` -- one `BarState` per monitor; `Tiler::mark_maximized()` -- the bars over a maximized or fullscreen window; the windows on screen that start or stop filling their monitor (`bar_filling.rs`) |
| `crates/mosaico-windows/src/bar/mod.rs` | `Win32BarRenderer` |
| `crates/mosaico-windows/src/bar/window.rs` | `Bar` -- per-monitor overlay window, its class and window procedure |
| `crates/mosaico-windows/src/bar/surface.rs` | `Bar` as a `BarSurface`: draw, hide, click-through |
| `crates/mosaico-windows/src/bar/input.rs` | `BarInput` -- what the window procedure reports to the sink, and the pointer's shape |
| `crates/mosaico-windows/src/bar/input_messages.rs`, `input_os.rs` | What the pointer messages say, and the calls the window makes for the pointer |
| `crates/mosaico-windows/src/bar/render.rs`, `canvas.rs` | DIB rendering pipeline, on a `Canvas` |
| `crates/mosaico-windows/src/bar/pixels.rs` | `leave_no_hole()` -- no fully transparent pixel in a clickable item while the bar takes the pointer |
| `crates/mosaico-windows/src/bar/text.rs` | `DrawCtx`, text and pill rendering helpers |
| `crates/mosaico-windows/src/bar/widgets/*.rs` | Draws the items: pills with their line icons, separators, workspaces, window icon, and says where each went |
| `crates/mosaico-windows/src/daemon_loop_bar.rs` | `ChannelSink` -- puts what the bars report on the daemon's queue -- and the actions it asks for |
| `crates/mosaico-windows/src/system_info.rs` | `Win32SystemInfo` -- CPU, memory and time from Win32 |
| `crates/mosaico-windows/src/tiling/on_screen.rs` | `Desktop::windows_on_screen()` on Win32 -- the windows the bars look through for a maximized or fullscreen one |

### Key Types

- `BarManager` -- decides which monitors get a bar, the space it reserves
  and what each one shows; owns the config, the resolved colors and the
  CPU tracker, and draws through a `BarRenderer`
- `BarContent` / `BarItem` -- the items one bar draws on each side: the
  workspace numbers, the window icon, pills with a label, an optional
  line icon (`BarIcon`) and colors, and separators
- `BarState` -- what one monitor's bar shows of the tiler and the daemon
- `SystemReadings` -- CPU, memory, media and time, the same for every bar
- `SystemInfo` -- trait for the raw machine readings a platform reports
- `BarRenderer` / `BarSurface` -- traits to make, draw, hide, switch and
  drop a bar
- `BarId` -- a bar's monitor and the generation of the bars it was made
  with, which goes up every time the bars are made again
- `PointerSink` -- where a bar reports a `PointerEvent`: a `Press` or
  `Release` of a `Button` (`Left`, `Right`, `Middle`), a `Wheel` turn in
  notches, or `Leave`, with positions in bar coordinates
- `ItemRegion` -- where a bar drew an item, its `BarTarget` (a workspace
  label, or a widget by its `WidgetRef`, the side and index of its entry in
  the config) and whether it is clickable; `hit()` finds the one the pointer
  is over
- `Press` -- turns presses and releases into clicks; `WheelSteps` adds wheel
  turns up
- `BarMouseConfig` -- `enabled`, `scroll` and the `click` entries from the
  `[mouse]` section
- `ClickBinding` -- a `[[mouse.click]]` entry as the file writes it: its
  `widget`, `button` and `action`, kept as text, with `None` for a key it
  leaves out; `ClickBinding::NONE` is the action that removes a binding
- `WidgetClicks` -- the click bindings over the defaults:
  `action(kind, side, button)`, `clickable(kind, side)` and `len()`;
  `rejected()` lists the entries it leaves out, as `RejectedClick`s with a
  `ClickProblem`
- `BarConfig` -- full bar configuration from `bar.toml`
- `BarColors` -- 9 color fields for all bar UI elements
- `WidgetConfig` -- tagged enum for widget type and settings; `kind()` gives
  its `WidgetKind`
- `Bar` -- a single monitor's status bar on Windows; wraps a layered Win32
  `HWND`
- `DrawCtx` -- GDI drawing context bundling DC, pixel buffer, and dimensions

## Platform Contracts

A platform gives the bar two things:

- `SystemInfo` reports raw readings: processor time counters, the share of
  memory in use, what is playing and the local time. Each call must return
  at once, since the daemon makes them on its own thread every second.
  `CpuTracker` turns two counter readings into a percentage.
- `BarRenderer::create(id, area, sink)` makes a hidden bar at the top of a
  monitor's work area, letting the pointer through. The `BarSurface` it
  returns draws a `BarContent` in the config's style and shows it, and
  returns where it drew each item (`ItemRegion`s, from
  `BarContent::region()` and `label_region()`, across the bar's full
  height); hides; switches between letting the pointer through and taking
  it (`set_click_through`); and is removed when dropped. A bar never takes
  focus, whatever the pointer does on it, stays out of the taskbar and task
  switcher, and is never reported to the tiler as a window.
- While a bar takes the pointer, it takes it over each clickable item, whole,
  and wherever it draws something, and lets it through where it draws
  nothing. It reports presses, releases, wheel turns and the pointer leaving
  to `sink` under `id`, holding the pointer from a press to its release.
  It shows a hand over the items `hit()` finds clickable in the regions it
  last returned. A platform that cannot take pointer input keeps the
  default `set_click_through`, which does nothing: its bars always let the
  pointer through, and report nothing.

The full contracts are on the traits in `crates/mosaico-core/src/bar/`.

## Window Properties

Each bar is a Win32 window with:

- `WS_POPUP` -- no frame
- `WS_EX_LAYERED` -- per-pixel alpha transparency
- `WS_EX_TOOLWINDOW` -- hidden from taskbar and Alt+Tab
- `WS_EX_NOACTIVATE` -- a click on it never activates it, and the window
  procedure answers `WM_MOUSEACTIVATE` with `MA_NOACTIVATE` too
- `WS_EX_TRANSPARENT` -- click-through, from the start and whenever core
  asks for it; removed while the bar takes the pointer

Each draw moves it to the top of the z-order (`HWND_TOP`) without
activating it (`SWP_NOACTIVATE`).

While it takes the pointer, the window procedure (`input.rs`) reports
`WM_LBUTTONDOWN`/`UP`, `WM_RBUTTONDOWN`/`UP` and `WM_MBUTTONDOWN`/`UP` with
their client coordinates, takes the pointer with `SetCapture` on a press and
gives it back with `ReleaseCapture` once no button is down, and reports
losing it before then (`WM_CAPTURECHANGED`) as the pointer leaving.
`WM_MOUSEWHEEL` and `WM_MOUSEHWHEEL` carry screen coordinates, which
`ScreenToClient` converts, and a delta in 120ths of a notch, turned into
notches, positive down or to the right. The first `WM_MOUSEMOVE` asks
`TrackMouseEvent` for `WM_MOUSELEAVE`, reported as the pointer leaving unless
a press holds the pointer; moves themselves are not reported. `WM_SETCURSOR`
shows `IDC_HAND` over a clickable item and `IDC_ARROW` elsewhere. The window
procedure reaches the bar's `BarInput` through the window's user data, which
the bar clears before it destroys the window.

Window class `MosaicoBar` is registered once. `WINEVENT_SKIPOWNPROCESS`
prevents bar windows from triggering tiling events. The window procedure
ignores `WM_CLOSE`, so End task cannot destroy a bar that `BarManager`
still holds.

## Rendering Pipeline

`render_bar()` executes the full rendering pass:

1. Creates a `Canvas`: a compatible DC and a 32-bit BGRA DIB section
2. Fills background with configurable opacity (premultiplied alpha)
3. Creates a GDI font from the configured font name and size
4. Builds a `DrawCtx` with the DC, pixel buffer, and dimensions
5. Calls `widgets::draw()`, which draws the content's left items, then its
   right items, and returns where each went
6. While the bar takes the pointer, gives every pixel left fully
   transparent inside a clickable item an alpha of 1 (`leave_no_hole()`),
   since a layered window lets the pointer through those: there, the
   background of a bar with `background_opacity = 0`, and the edges of
   glyphs that GDI draws a pixel past the text it measures, with no alpha.
   No one can see the difference. The rest of the strip keeps the alpha it
   was painted with, so the pointer goes through where the bar draws nothing
7. Calls `UpdateLayeredWindow` with `ULW_ALPHA` to atomically present

Steps 2 to 6 are `paint()`, which the tests run on a canvas of their own,
with no window on screen.

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
  be missing. When the icon is one of the glyphs `BarIcon` has a line
  drawing for (grid, window, clock, calendar, memory, processor, download,
  pause, music, columns), the pill carries the drawing instead and the
  label is the text alone. Every platform paints the drawing with
  `bar::mask`, in the pill's text color, as tall as the font's em, so the
  bar looks like the settings preview with or without a Nerd Font. Any
  other icon stays in the label as text.
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
- `wrap` -- `[workspaces] wrap`, which the wheel steps with
- `maximized` -- whether a maximized or fullscreen window is under the bar,
  which then lets the pointer through

`Tiler::bar_states()` builds one per monitor on each render cycle, and
`TilingManager::bar_states()` passes it the update text and pause state, and
marks the monitors with a maximized or fullscreen window
(`Tiler::mark_maximized()`, see [Click-Through](#click-through)).

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

- `new(renderer, system, sink, config, monitor_rects, theme)` -- creates
  one bar per selected monitor through the renderer, each with its `BarId`
  and the `sink` it reports the pointer to, resolves colors from theme,
  starts a `CpuTracker` when a `cpu` widget is configured, and logs each
  `[[mouse.click]]` entry it leaves out (see [Widget Clicks](#widget-clicks))
- `update(states)` -- reads the system once, then switches each bar to let
  the pointer through or take it, as [Click-Through](#click-through) says,
  builds its content from its monitor's state and draws it, keeping where
  its items went, and whether the tiler has a state for its monitor; does
  nothing while no monitor has a bar
- `pointer(bar, event, now)` -- what the pointer did on a bar, and the action
  it asks for, if any; `wheel_due()` and `take_wheel(now)` -- the wheel's
  turns, once they are due (see [Pointer Input](#pointer-input))
- `bar_height()` -- returns the bar height while at least one monitor has a
  bar, 0 otherwise, so a bar that no monitor gets reserves no space (the
  tiler reads an empty monitor list as every monitor)
- `bar_monitor_indices()` -- returns which monitor indices have bars
- `resolve_colors(theme)` -- re-resolves every color on theme change, the
  `[colors]` table and each widget's `color`
- `rebuild_for_monitors(monitor_rects, theme)` -- replaces the bars after a
  display change
- `reload(config)` -- recreates bars with new config, returns new height,
  and logs each `[[mouse.click]]` entry the new config has that it leaves out
- `hide_all()` -- hides all bars on shutdown

Bars made again, by a reload or a display change, are of a new generation,
so their ids are never those of the bars they replace.

A monitor whose bar cannot be made goes without one, and the failure is
logged. The manager keeps the config as loaded (`raw_config`) apart from
the resolved one, so a theme change resolves every color again from what
`bar.toml` says: a color given by name follows the new theme, and an empty
one takes the new theme's default. The daemon creates the manager with
`Win32BarRenderer` and `Win32SystemInfo`.

## Pointer Input

With `[mouse] enabled = true`, the default, the bar takes clicks and the
wheel:

| Where | What | Result |
|-------|------|--------|
| A workspace number (left side) | left click | Go to that workspace on the bar's monitor, every monitor in global mode |
| On the bar, where it takes the pointer | wheel | The workspace before (up, left) or after (down, right) on the bar's monitor, with `scroll = "workspaces"` |
| The layout widget | left / right click | Cycle the layout / toggle monocle on the bar's monitor (defaults) |
| The paused widget | left click | Resume the hotkeys (default) |
| Any widget a `[[mouse.click]]` entry names | that button | The entry's action: on the bar's monitor, or on the whole daemon for `retile` and `toggle-pause` |
| A clickable item | hover | The pointer shows a hand; elsewhere on the bar, the arrow |

### Widget Clicks

A click on a widget runs what its button is bound to on that kind of widget.
The bindings are the `[[mouse.click]]` entries of `bar.toml` over the
defaults (`ClickBinding::defaults()`), merged by `WidgetClicks`:

| Widget | Button | Action |
|--------|--------|--------|
| `layout` | left | `cycle-layout` |
| `layout` | right | `toggle-monocle` |
| `paused` | left | `toggle-pause` |

- An entry names the widget by its type, as `[[left]]` and `[[right]]` do,
  the button, `left`, `right` or `middle`, and the action, as
  `keybindings.toml` names them. It replaces the binding of its widget and
  button, or adds one, and of two entries for the same widget and button the
  later one wins. A binding holds for every widget of its type.
- An entry with `action = "none"` (`ClickBinding::NONE`) removes the binding
  of its widget and button, a default included, as `scroll = "none"` turns
  the wheel off. A widget it leaves with no binding takes no clicks: no hand,
  and no least alpha. Such an entry is not a mistake, so it is neither
  logged nor flagged, and a later entry for the same widget and button still
  wins over it, as it wins over an earlier one. Deleting an entry, or
  writing `click = []`, keeps the defaults.
- A bar may only run an action that acts on a monitor, its own
  (`cycle-layout`, `toggle-monocle`, `goto-workspace-N`,
  `goto-workspace-next`, `goto-workspace-prev`), or on the whole daemon
  (`toggle-pause`, `retile`), as `Action::scope()` says. An action about the
  focused window (`close-focused`, `minimize-focused`, `send-to-workspace-N`,
  `focus-*`, `move-*`) would act on a window of another monitor when the bar
  of one is clicked. An entry that names one, that has no widget, button or
  action, or that names one that does not exist, is left out as if the file
  did not have it, so the binding of its widget and button stays, and the
  rest of `bar.toml` applies all the same. `WidgetClicks::rejected()` says
  which entries and why, with the keys each one has: the bar manager logs
  them as it takes the config, and `mosaico doctor` lists them, since
  logging is off by default.
- A workspace label is part of the workspaces widget. Its left click always
  goes to its workspace, which is not a binding, and its right and middle
  clicks run the workspaces widget's bindings, as any click between the
  labels does. The workspaces widget on the right, a plain pill with its
  icon, takes no clicks.
- A widget with a binding is clickable (`BarContent::region()`), so the
  pointer shows a hand over it, and the bar takes the pointer over its whole
  column of the bar, above and below its pill included, where it draws
  nothing.
- `toggle-pause` pauses the hotkeys only, so the paused widget, which shows
  while they are paused, is how the mouse resumes them. A click on the bar
  of a monitor the tiler has no state for runs nothing, not even an action
  on the whole daemon.

### From a Click to an Action

1. The bar reports to its `PointerSink`, which in the daemon is a
   `ChannelSink` that puts each report on the daemon's queue as
   `DaemonMsg::BarPointer`. The loop runs it as a task, in order with the
   hotkeys and commands, against the state the bar shows, and it counts as
   input, as a key does. Hovering is not reported, so it never does.
2. `BarManager::pointer(bar, event, now)` drops a report whose `BarId` is
   not one of the bars on screen: what the pointer did on a bar made again
   since, after a reload or a display change, never lands on its successor,
   and a press on a bar that is gone never pairs with a release on the new
   one.
3. A press and a release of the same button on the same item are a click
   (`Press`). The item is found with `hit()` in the regions the bar's last
   draw returned, the one drawn last on top, so the right side wins where
   the sides overlap. A click names what it hit, a workspace or a widget's
   entry in the config, never its place in the row. A release on another
   item, in a gap or off the bar clicks nothing, and so does any click on a
   bar whose last draw failed, or that is hidden.
4. A left click on a workspace label asks for `GoToWorkspace(n)` from
   `Origin::Bar(monitor)`. Any other click asks for the action its button
   runs on the kind of widget clicked, from the same origin (see
   [Widget Clicks](#widget-clicks)), and on a bar whose monitor the tiler had
   no state for when it was drawn, for nothing.
5. Wheel turns add up in `WheelSteps`, fractions of a notch from a touchpad
   or a high-resolution wheel included, in both directions. Once
   `WHEEL_QUIET` (120 ms) passed since the last turn, `take_wheel(now)`
   asks for one `GoToWorkspace`, to the workspace that many whole steps
   from the one the bar shows, stepping as `goto-workspace-next` does with
   the `wrap` of its `BarState`. A fraction of a notch left over adds up
   with the next turns, unless they go the other way or come a second or
   more after the last one, or the bars are made again. A press on the bar
   drops the turns waiting there, and so does a switch of its monitor made
   some other way meanwhile, such as with a key: they were turned from the
   workspace the bar showed as they began. With `scroll = "none"` the wheel
   does nothing.
6. The daemon carries the action out with `carry_out()`, the way it carries
   out a hotkey's, with its origin, so it honours the workspace mode.
   `toggle-pause` is handled there, whichever bar asks. Pausing pauses the
   hotkeys only: the bar keeps working.

### Focus and the Pointer

- A click on the focused monitor's bar behaves like the key: the
  workspace's last focused window takes focus, or focus parks on the
  monitor when the workspace is empty.
- A click on another monitor's bar that acts on a monitor, switching its
  workspace or cycling its layout or monocle, moves focus to that monitor
  first, to the window its workspace on screen takes focus back with, or
  parks it there, then behaves the same. The workspace focus left
  remembers its window, for coming back. `retile` and `toggle-pause` act
  on the whole daemon, so a click that runs one leaves focus where it is,
  on whichever monitor's bar it lands.
- In global mode every monitor switches, and focus goes to the clicked
  monitor.
- Nothing a click or a scroll does moves the pointer, whatever
  `[mouse] follows_focus` says.
- While focus is parked on an empty workspace, a pointer on a bar counts as
  pointing at that bar's monitor (`monitor_at_point`), so the claim takes a
  focus report there for the user's own.
- A bar of a monitor the tiler has no state for does nothing, in global mode
  too.

### Click-Through

A bar lets the pointer through to the windows under it, as it always did,
while `[mouse] enabled = false`, and while a maximized or fullscreen window
is under it. `TilingManager::bar_states()` marks the monitor from
`Desktop::windows_on_screen()`: the windows shown, not minimized and not
kept out of view, managed or not, which leaves out the desktop itself, the
shell's surfaces such as the taskbar, windows the pointer goes through, and
the bars. One of them counts when it is on the monitor and maximized or over
the whole of the monitor's work area, but a window the tiler manages counts
only while its workspace is on screen, whatever the desktop reports, so one
the tiler hid never does. `BarManager::update()` switches a bar only when
that changes, before it draws it, and asks again on the next refresh when
the switch fails. On Windows a window lets the pointer through when it is
layered and `WS_EX_TRANSPARENT`. An overlay that lets it through by its
pixels' alpha alone, without that style, and covers a monitor, keeps that
monitor's bar click-through for as long as it shows.

Maximizing, restoring and going fullscreen only report that the window
moved. The tiler notes each window on screen that starts or stops filling
its monitor that way (`TilingManager::take_bars_changed()`), and the daemon
draws the bars again at once. Anything else that changes what is under a bar
without an event of its own shows at the next refresh, a second later at
most.

While a bar takes the pointer, it takes it over the whole column of the bar
each clickable item stands in, from the top of the strip to the bottom, and
wherever it draws something. Elsewhere, where a pixel stays fully
transparent, the pointer goes through to the window under it. With the
default `background_opacity = 0`, a window that only reaches into the
strip, such as a Snap half or a floating window moved to the top, keeps its
clicks there except over the bar's labels and pills and the columns of its
clickable items. A bar with any `background_opacity` above 0 takes the
pointer over all of its strip, which it covers: even 1 paints it with an
alpha of 2.

So once a bar is drawn after a maximized or fullscreen window came under
it, it takes none of that window's clicks, and over any other window it
takes clicks only over the columns of its clickable items and where it
draws.

### Timing

The bars' window procedures run when the daemon drains its message queue, at
the top of each loop iteration, so a click can wait up to `PUMP_INTERVAL`
(50 ms) and can run after a key pressed just after it. The scheduler ends
the loop's wait when the wheel's turns are due, and runs the wheel step then
(see [daemon.md](daemon.md#main-loop)), so the switch comes `WHEEL_QUIET`
after the last turn.

### The Wheel and the Windows Settings

Windows sends the wheel to the window under the pointer only while "Scroll
inactive windows when I hover over them" is on, as it is by default
(Settings > Bluetooth & devices > Mouse). With it off, the wheel goes to the
window that has keyboard focus, which the bar never has, so scrolling over
the bar does nothing. Clicks work either way.

### On the Live Desktop

`crates/mosaico/tests/bar_click.rs` clicks and scrolls the bar with the real
pointer, on a daemon of its own that draws the bar from a `bar.toml` of its
own, so it runs on demand with the desktop suites, never with the safe
ones. It finds the workspace labels and the layout widget on the primary
monitor's bar the way the user does, left to right where the pointer shows
the hand, and presses or turns the wheel only over one of them, with nothing
in front of the bar there. A click on label 2 goes to workspace 2 and one on
label 1 comes back, and a notch of the wheel down steps to workspace 2 and
one up steps back, as `debug state` shows. Three clicks on the layout widget
go round the three layouts, read from the widget's width, since
`debug state` does not show layouts. It checks the setting above before it
turns the wheel, and once a click or a turn ends, or fails half way, the
button is up and the pointer is back where it was.

## Work Area Reservation

The daemon subtracts the bar height from each monitor's work area before
passing it to the `TilingManager`. This ensures BSP layout computation
accounts for the bar space.

- `adjust_work_areas_for_bar(bar_height, bar_monitors)` -- offsets
  `work_area.y` and `work_area.height` for selected monitors
- `reset_and_adjust_work_areas(bar_height, bar_monitors)` -- resets to
  OS values first to prevent offset accumulation on config reload

## Refresh Strategy

- **Tiling state changes**: bars re-render on every action, workspace
  switch and window event, but for a window's report that it moved, which
  only re-renders them when a window on screen went in or out of maximized
  or fullscreen
- **System widgets**: a 1-second `DaemonMsg::Tick` triggers clock, RAM,
  CPU, and date refresh
- **Config reload**: `ConfigReload::Bar` recreates bars, resets work areas

## Per-Monitor Filtering

The `monitors` field in `bar.toml` controls which monitors display bars.
An empty list (default) shows bars on all monitors. Specifying monitor
indices (0-based) restricts bars to those monitors only.

## Configuration

See [configuration.md](configuration.md) for the full `bar.toml` format.
What the mouse does is the `[mouse]` section:

```toml
[mouse]
enabled = true           # false keeps the bar click-through, as before
scroll = "workspaces"    # what scrolling over the bar does: "workspaces" or "none"

# Clicks on widgets run actions, named as in keybindings. These are the
# defaults: deleting an entry keeps its default, a later entry for the same
# widget and button wins, and action = "none" removes the binding.
[[mouse.click]]
widget = "layout"
button = "left"          # left, right or middle
action = "cycle-layout"

[[mouse.click]]
widget = "layout"
button = "right"
action = "toggle-monocle"

[[mouse.click]]
widget = "paused"
button = "left"
action = "toggle-pause"
```

A `bar.toml` without the section gets a comment block about it at the next
start, which changes nothing it says (see
[configuration.md](configuration.md#files-a-newer-version-updates)). The
block tells of `enabled`, `scroll`, `[[mouse.click]]` and `action = "none"`,
and names no default.

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
- **The bar runs inside the daemon**, so a click is input on the daemon's own
  loop, against the state the bar shows. Nothing crosses a process boundary,
  and the bar cannot fall out of step with the tiler.
- **Clicks by identity and generation**: a region names the workspace or the
  widget's entry it shows, and a bar's id its generation, so hidden or
  reordered items never map to the wrong workspace, and what the pointer did
  on a bar made again since is dropped instead of landing on something else.
- **The pointer only where the bar draws**: a layered window passes the
  pointer through fully transparent pixels, so only the clickable items are
  given the least alpha, which keeps the pointer on them. A bar never takes
  a click under its empty, transparent strip, and the windows there keep
  theirs.
- **The platform only reports**: what was clicked, what a scroll is worth,
  which monitor it acts on and whether the bar lets the pointer through are
  decided in core and tested against a fake renderer. The platform answers
  the pointer's shape with core's `hit()` on the regions it drew.
- **Click bindings checked one at a time**: `ClickBinding` keeps the names of
  an entry as text, checked as the bar takes the config, so an entry with an
  unknown or missing widget, button or action is left out on its own, and
  the rest of `bar.toml` applies. The same holds for key names in
  `keybindings.toml`. A value of the wrong type, such as `button = 1`, or
  `[mouse.click]` with single brackets, still makes `bar.toml` invalid, as
  any other mistake there does.
- **Bindings by widget type**: an entry names a type, not an entry of
  `[[left]]` or `[[right]]`, so it holds for a widget the user moves to the
  other side or adds again, and the defaults need no place in the lists.

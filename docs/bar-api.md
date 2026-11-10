# Bar API

Every bar mosaico shows its state on follows one contract, whichever program
draws it. The built-in bar (see [status-bar.md](status-bar.md)) is the first
to follow it: it reads mosaico only through the state described here, so a
bar another program draws can show everything the built-in one shows.

The contract has two halves:

- **The state** (`mosaico_core::api`): what a bar is shown of mosaico, as
  plain data that reads and writes as JSON, and the updates between two
  states.
- **The provider** (`mosaico_core::bar::BarProvider`): what the daemon asks
  of the bar it runs, and in what order.

## Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/api/state.rs` | `State` and its topics: `Daemon`, `Theme`, `Monitor`, `Workspace`, `Focus`, `FocusedWindow`, `Rect` |
| `crates/mosaico-core/src/api/diff.rs` | `Update`, `Removed` -- `State::diff()` and `State::apply()` |
| `crates/mosaico-core/src/api/mod.rs` | `API_VERSION`, `MAX_TITLE_CHARS` |
| `crates/mosaico-core/src/tiler/api_state.rs` | The tiler's monitors, focus and workspace settings as the API shows them |
| `crates/mosaico-core/src/tiler/manager_api.rs` | `TilingManager::bar_state()` -- the state, built again only when it may have changed |
| `crates/mosaico-core/src/tiler/reach.rs` | `MonitorPlace`, `Reach` -- where a bar is placed, and how far it reaches |
| `crates/mosaico-core/src/bar/provider.rs` | `BarProvider`, `NoBar` |
| `crates/mosaico-core/src/bar/host.rs` | `BarHost` -- the daemon's one bar, and the order the contract asks for |
| `crates/mosaico-core/src/bar/builtin.rs` | `BuiltinBar` -- the built-in bar as a provider |
| `docs/bar-api/state.json`, `update.json` | An example state and update |
| `docs/bar-api/state.schema.json`, `update.schema.json` | Their JSON schemas |

## The State

A `State` is grouped by topic, so a bar that shows only the workspaces
never needs to look at the theme:

| Topic | What it holds |
|-------|---------------|
| `daemon` | `version`, the `api` version, the `instance` name (`null` for the default instance), the `provider` the user picked, whether the hotkeys are `paused`, the `update` notice (`null` when up to date), `workspace_mode` (`"per-monitor"` or `"global"`) and `wrap` |
| `theme` | The theme's `name` and `flavor`, and its `colors` by name as `#rrggbb` |
| `monitors` | Every connected monitor, from the left (see below) |
| `focus` | The `monitor` and `workspace` the user works on, and the focused `window`: its `id`, `title` and `program` |

Each monitor has:

| Field | Meaning |
|-------|---------|
| `id` | Its `StableMonitorId`, kept through display changes and reconnects (see [monitor-management.md](monitor-management.md#stable-monitor-ids)) |
| `index` | Position from the left, from 0, as `bar.toml`'s `monitors` counts |
| `name`, `device`, `scale` | What the user calls the display, the name other programs know it by, and its scale factor (1.0 at 100%) |
| `bounds` | The whole monitor, in the desktop's pixels |
| `work_area` | What the system leaves for windows |
| `tiling_area` | Where mosaico tiles: the work area, kept clear of the bar |
| `maximized` | Whether a maximized or fullscreen window covers the monitor, so a bar can step aside for it |
| `active_workspace` | Number of the workspace on screen, from 1 |
| `workspaces` | Each workspace in number order: `number`, `name`, `layout` (`"bsp"`, `"vertical-stack"` or `"three-column"`), `monocle`, its `windows` in layout order then the floating ones, and the one `focused` last |

[`bar-api/state.json`](bar-api/state.json) is a complete example.

Rules a reader can rely on:

- `monitors` is always in `index` order.
- Window ids are only good for the session.
- A title is cut at `MAX_TITLE_CHARS` (1000) characters. Show it as text,
  never as markup: it is whatever the window says.
- A layout a bar does not know is shown as is; the built-in bar shows it as
  the default layout.

## Updates

`older.diff(&newer)` is an `Update`: each topic that changed, in full, each
monitor that changed or was connected, in full, by id, and the ids of the
monitors that were disconnected (`removed.monitors`). Applying one is
replacing objects by id, so a bar keeps no model of its own: applying
`older.diff(&newer)` to `older` (`State::apply()`) gives `newer`, which
property tests check on random states. A field that did not change is left
out of the JSON.

[`bar-api/update.json`](bar-api/update.json) is an example.

## Versions

`daemon.api` is `API_VERSION`, 1 today. A change that only adds fields
keeps the version: every object reads with the fields it lacks at their
defaults and ignores the fields it does not know, which a test checks. A
change that removes a field or changes what one means raises it.

The schemas in `docs/bar-api` are generated from the types, and the
examples read and write back byte for byte, both checked by
`crates/mosaico-core/src/api/tests/examples_tests.rs`. After changing the
types on purpose, run those tests with `MOSAICO_BLESS_API=1` to write the
files again, and review the diff.

## Building the State

`TilingManager::bar_state(daemon, theme)` returns the state when it differs
from what it returned last, or on the first call, and `None` otherwise;
`last_bar_state()` is the last one. The daemon's own facts (`version`,
`paused`, `update`, ...) and the theme come in as arguments; the tiler fills
in the workspace settings, the monitors and the focus.

The desktop is asked only when a step since the last call may have changed
what a bar shows:

- Every input but a window's report that it moved, or that a drag started,
  marks the state as maybe changed. Those come on every frame of a drag or
  an animation, and change nothing a bar shows unless a window on screen
  starts or stops filling its monitor, which the tiler notes
  (`Tiler::take_bars_changed()`).
- Which monitors have a maximized or fullscreen window is read from
  `Desktop::windows_on_screen()`, as the tiler sees it: a window the tiler
  manages counts only while its workspace is on screen.
- The focused window's title and program are read only when the focus
  moves, or when that window reports a new title.

The metrics scenarios count these reads (`read_windows_on_screen`,
`read_facts` in `metrics/tiler.toml`), showing the state once after each
action as the daemon does.

## The Provider Contract

```rust
pub trait BarProvider {
    fn name(&self) -> &str;
    fn place(&mut self, places: &[MonitorPlace]);
    fn reach(&self, places: &[MonitorPlace]) -> Reach;
    fn show(&mut self, state: &State);
    fn refresh(&mut self) {}
    fn pointer(&mut self, bar: BarId, event: PointerEvent, now: Instant)
        -> Option<(Action, Origin)> { None }
    fn due(&self) -> Option<Instant> { None }
    fn take_due(&mut self, now: Instant) -> Vec<(Action, Origin)> { Vec::new() }
    fn reload(&mut self, config: BarConfig) {}
    fn stop(&mut self);
}
```

The daemon owns one provider at a time and is its only caller, on its own
thread:

- `place` comes before the first `show`, and again whenever the monitors or
  their work areas change. A provider that draws makes or moves its bars
  there. `MonitorPlace` gives each monitor's id, index, bounds, work area
  and scale.
- `reach` is how far the bars reach into each monitor as they are placed
  now. It must not change between two calls to `place` unless the bar's
  own config changed, since the tiler only asks for it then.
- `show` is only called with a state that differs from the one shown last,
  or after `place`. A provider keeps the last state if it needs to draw
  again on its own.
- `refresh` is called about once a second when the state did not change,
  for what changes on its own, such as a clock, so a bar draws once a tick.
- `pointer`, `due` and `take_due` are for a provider drawn through
  mosaico's renderer: what the pointer did on it, and the actions it hands
  over on its own, such as a workspace switch once the wheel went quiet.
- `reload` takes a new `bar.toml`; a provider that does not draw ignores it.
- `stop` takes the bars off screen.
- Nothing may block: a provider talking to another program queues what it
  sends and drops it when that program falls behind.

### BarHost

The daemon holds no bar of its own, only a `BarHost`, which runs the
provider and owns the order above, so the order holds whichever provider
runs, and is tested in core (`tiler/tests/bar_host_tests.rs`):

| Call | When | What it does |
|------|------|--------------|
| `start` | Startup | Place, keep the tiles clear of the reach, show |
| `show` | After each action, batch of window events and reload | Show the state when it changed, or when the bars were placed since |
| `tick` | Once a second | `show`, or else `refresh` |
| `display_changed` | A display change settled | Place on the monitors as they will be, then have the tiler rebuild them clear of the reach, in one step |
| `reset_work_areas` | The system's work areas changed, or `retile` | Read the work areas, place on them, keep clear of the reach, in one retile |
| `reload` | `bar.toml` changed | The provider takes the config, then `reset_work_areas` |
| `set_theme`, `set_paused`, `set_update` | The theme, pause state or update notice changed | Shown from the next `show` |

`daemon.provider` is the name the provider goes by.

### Providers

- `BuiltinBar` (`"mosaico"`): the built-in bar. It reads one `BarState` per
  monitor from the state (`BarState::from_state()`), follows the theme the
  state names, makes its bars again only when a monitor came, went, moved or
  changed size, work area or scale, and reaches its height down from the
  top of the work area on the monitors `bar.toml` picks.
- `NoBar` (`"none"`): nothing is shown and nothing kept clear.

A parity test (`tiler/tests/bar_parity_tests.rs`) draws the built-in bar
from the built state and from its JSON round trip, and checks they draw the
same.

## Reach

`Reach` is how far the active bar reaches into each monitor, which tiles
keep clear of:

- Monitors are named by `StableMonitorId`, so a reach means the same
  display after a display change.
- A reach is measured from the monitor's edge, not from the system's work
  area, and tiles keep clear of whichever of the two reaches further, so a
  bar never needs to know what the system keeps.
- Only the top edge is used today. A monitor with no entry has nothing kept
  clear.

`Reach::bar_on(places, height, indices)` is the reach of a bar `height`
pixels tall at the top of the work area, on the monitors at `indices`, or
on every one when `indices` is empty.

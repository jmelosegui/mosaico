# Bar API

Every bar mosaico shows its state on follows one contract, whichever program
draws it. The built-in bar (see [status-bar.md](status-bar.md)) is the first
to follow it: it reads mosaico only through the state described here, so a
bar another program draws can show everything the built-in one shows.

The contract has three parts:

- **The state** (`mosaico_core::api`): what a bar is shown of mosaico, as
  plain data that reads and writes as JSON, and the updates between two
  states.
- **The provider** (`mosaico_core::bar::BarProvider`): what the daemon asks
  of the bar it runs, and in what order.
- **The sessions** (`mosaico_core::api`): how a bar another program draws
  reads the state and runs actions, over a pipe of its own (see
  [Sessions](#sessions)).

## Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/api/state.rs` | `State` and its topics: `Daemon`, `Theme`, `Monitor`, `Workspace`, `Focus`, `FocusedWindow`, `Rect` |
| `crates/mosaico-core/src/api/diff.rs` | `Update`, `Removed` -- `State::diff()` and `State::apply()` |
| `crates/mosaico-core/src/api/mod.rs` | `API_VERSION`, `MAX_TITLE_CHARS` |
| `crates/mosaico-core/src/tiler/api_state.rs` | The tiler's monitors, focus and workspace settings as the API shows them |
| `crates/mosaico-core/src/tiler/manager_api.rs` | `TilingManager::bar_state()` -- the state, built again only when it may have changed |
| `crates/mosaico-core/src/tiler/reach.rs` | `MonitorPlace`, `Reach` -- where a bar is placed, and how far it reaches |
| `crates/mosaico-core/src/tiler/edges.rs` | `Edges`, `Side` -- how far something reaches in from each edge of a monitor |
| `crates/mosaico-core/src/bar/provider.rs` | `BarProvider`, `ApiBar`, `NoBar` |
| `crates/mosaico-core/src/config/bar_section.rs` | `[bar] provider` in `config.toml`: `BarChoice`, `BarSection`, `ProviderCheck` |
| `crates/mosaico-core/src/config/bar_reserve.rs` | `[bar.reserve]`: `BarReserve`, `ReserveEdges` |
| `crates/mosaico-core/src/bar/reserve.rs` | `ReservedSpace`, `reserve_reach()` -- what `[bar.reserve]` keeps on each monitor |
| `crates/mosaico-core/src/bar/handover.rs` | `Handover`, `HANDOVER` -- the old bar's space, held while another program's bar starts |
| `crates/mosaico-core/src/bar/host.rs` | `BarHost` -- the daemon's one bar, and the order the contract asks for |
| `crates/mosaico-core/src/bar/builtin.rs` | `BuiltinBar` -- the built-in bar as a provider |
| `docs/bar-api/state.json`, `update.json` | An example state and update |
| `docs/bar-api/state.schema.json`, `update.schema.json` | Their JSON schemas |
| `crates/mosaico-core/src/api/protocol.rs` | What a bar sends: `ClientMessage`, `Hello`, `Request`, `Run`, `Subscribe`; `negotiate()`, `SUPPORTED_APIS` |
| `crates/mosaico-core/src/api/messages.rs` | What the daemon sends: `DaemonMessage`, `Welcome`, `Snapshot`, `Ack`, `ApiError`, `ErrorCode`, `Bye` |
| `crates/mosaico-core/src/api/topics.rs` | `Topic`, `Topics` -- what a session follows, and the state cut down to it |
| `crates/mosaico-core/src/api/connection.rs` | `ApiListener`, `ApiConnection`, `ApiReader`, `ApiWriter` -- the contract for the platform's endpoint; `Limits` |
| `crates/mosaico-core/src/api/session.rs` | `serve_api()` -- accepts bars, opens sessions on `hello` and reads their requests |
| `crates/mosaico-core/src/api/hub.rs`, `outbox.rs` | `Hub` -- the latest state, the sessions and what each is owed |
| `crates/mosaico-core/src/api/writer.rs` | `SessionWriter` -- sends a session its replies and the state |
| `crates/mosaico-core/src/api/loop_request.rs` | `LoopRequest`, `Reply` -- what a session asks of the daemon loop |
| `crates/mosaico-core/src/api/client/` | `ApiClient`, `query()`, `run()`, `subscribe()` -- the bar's end of a session, as `mosaico query`, `subscribe` and `run` speak it; `JsonState`, `FieldPath`, `MonitorRef`, `Outcome` |
| `crates/mosaico-core/src/system/parent_watch.rs` | `ParentWatch` -- whether the program that started a command still runs |
| `docs/bar-api/client.jsonl`, `daemon.jsonl` | One example of each message, as sent |
| `docs/bar-api/client-message.schema.json`, `daemon-message.schema.json` | Their JSON schemas |

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
`crates/mosaico-core/src/api/tests/examples_tests.rs` and, for the
messages, `protocol_tests.rs`. After changing the types on purpose, run
those tests with `MOSAICO_BLESS_API=1` to write the files again, and review
the diff.

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
    fn show(&mut self, state: &State, now: Instant);
    fn wants_state(&self) -> bool { true }
    fn pointer(&mut self, bar: BarId, event: PointerEvent, now: Instant)
        -> Option<(Action, Origin)> { None }
    fn due(&self) -> Option<Instant> { None }
    fn wake(&mut self, now: Instant) -> Vec<(Action, Origin)> { Vec::new() }
    fn reload(&mut self, config: BarConfig) {}
    fn set_reserve(&mut self, reserve: &BarReserve) {}
    fn connects_later(&self) -> bool { false }
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
- `wants_state` says whether anything would come of `show` now. While it is
  `false` the daemon builds no state at all, and it shows the state it
  missed once it turns `true`.
- `due` is when the provider next needs `wake`: to draw again what
  changes on its own, such as a clock or a reading of the machine, or to
  hand over actions, such as a workspace switch once the wheel went quiet.
  The daemon has no other timer, so a provider with nothing waiting says
  `None` and is never woken for nothing. `wake(now)` does what is due and
  returns the actions.
- `pointer` is for a provider drawn through mosaico's renderer: what the
  pointer did on it.
- `reload` takes a new `bar.toml`; a provider that does not draw ignores it.
- `set_reserve` takes `[bar.reserve]` before `place`, at startup and on
  each reload that changes it. A provider whose bar cannot say how far it
  reaches keeps that space in `reach`; the others ignore it.
- `connects_later` says the bar is drawn by a program that may not be up
  yet, so on a switch the space the old bar kept is held until it is (see
  [The Handover](#the-handover)).
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
| `due`, `wake` | The loop waits no longer than `due`, then calls `wake` | `due` is the sooner of the provider's and the end of a handover. `wake` ends a handover whose time is up, retiling and showing once, then wakes the provider and returns its actions |
| `display_changed` | A display change settled | Place on the monitors as they will be, then have the tiler rebuild them clear of the reach, in one step |
| `reset_work_areas` | The system's work areas changed, or `retile` | Read the work areas, place on them, keep clear of the reach, in one retile |
| `reload` | `bar.toml` changed | The provider takes the config, then `reset_work_areas` |
| `choose` | `[bar] provider` or `[bar.reserve]` changed on a reload | Stop the old provider, make and place the new one, hold the old space if it connects later, retile only the monitors whose space kept changed, then show it the state. Under the same provider a new reserve retiles only the monitors it changed; nothing when neither changed |
| `set_theme`, `set_paused`, `set_update` | The theme, pause state or update notice changed | Shown from the next `show` |

`daemon.provider` is the name the provider goes by. `BarHost` is given the
choice and a maker for the built-in bar, so the platform decides how that
bar draws and core decides when it is made: at startup under `"mosaico"`,
and each time a reload picks it again, with `bar.toml` read as it is then.
Switching to a bar that keeps the same space moves no window, since the
tiler leaves a window where it already is.

### Providers

- `BuiltinBar` (`"mosaico"`): the built-in bar. It reads one `BarState` per
  monitor from the state (`BarState::from_state()`), follows the theme the
  state names, makes its bars again only when a monitor came, went, moved or
  changed size, work area or scale, and reaches its height down from the
  top of the work area on the monitors `bar.toml` picks.
- `ApiBar` (`"external"`): a bar another program draws. mosaico draws
  nothing and keeps tiles clear of what the system keeps and of
  `[bar.reserve]`, scaled to each monitor (`ReservedSpace::on()`). It wants
  no state until a program reads it through the API, and connects later.
- `NoBar` (`"none"`): nothing is shown and nothing kept clear, and no state
  is built.

`[bar] provider` in `config.toml` picks one (see
[configuration.md](configuration.md)). It is compared without regard to
case, and a name mosaico does not know is read as `"external"`, so the
built-in bar never draws over a bar the user picked.

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
- Each monitor has an `Edges`: how far the bar reaches in from its top,
  bottom, left and right. A monitor with no entry has nothing kept clear.
- `Reach::furthest()` keeps, per monitor and edge, the further of two
  reaches, as the handover does.

`Reach::bar_on(places, height, indices)` is the reach of a bar `height`
pixels tall at the top of the work area, on the monitors at `indices`, or
on every one when `indices` is empty.

### The Handover

A bar drawn by another program starts after mosaico switches to it, and
says how far it reaches only once it runs. Until then, the space the old
bar kept would be given to tiles, and taken back a moment later. So when
`choose` switches to a provider that `connects_later`, `BarHost` holds
the old reach (`Handover`) on every monitor and edge the new one keeps
nothing at, for at most `HANDOVER` (5 seconds):

- tiles keep clear of the further of the held reach and the new one;
- a monitor the new bar reaches into is released at once;
- once the time is up, `wake` drops what is left and retiles once;
  `BarHost::due` includes that time, so the loop wakes for it;
- once the new bar reaches into every monitor held, the hold ends early.

A switch from mosaico's bar to `"external"` therefore moves no window
while the other bar starts, and at most one retile once the hold ends.

## Sessions

A bar drawn by another program reads the state, and runs actions, in a
session on the daemon's bar API endpoint: on Windows the pipe
`\\.\pipe\mosaico.bar`, or `\\.\pipe\mosaico-<name>.bar` for a named
instance (see [ipc.md](ipc.md#the-bar-api-pipe)). Every message is one line
of compact JSON, as on the CLI's pipe, and at most 1 MiB. One example of
each is in `docs/bar-api/client.jsonl` (what a bar sends) and
`docs/bar-api/daemon.jsonl` (what the daemon sends), with their schemas
beside them. A bar ignores keys and messages it does not know, so later
versions may add some.

### Opening

The bar speaks first:

```json
{"hello":{"api":[1],"bar":"yasb","client":"mosaico-yasb 1.0","topics":["daemon","focus"]}}
```

- `api`: the versions the bar speaks. The daemon picks the highest it
  speaks too, or answers `unsupported-api` with the ones it does, and
  closes.
- `bar`: the provider the bar speaks for, if it is one mosaico knows.
- `client`: a name for the log.
- `topics`: what to follow, of `daemon`, `theme`, `monitors` and
  `focus`; all four when left out. Names the daemon does not know are left
  out, so a bar written for a later version still opens a session.

The daemon answers `welcome`, with the version, its `boot` (which changes
when the daemon restarts), the provider the user picked and the topics
followed, then a snapshot of them:

```text
{"welcome":{"api":1,"mosaico":"0.15.0","boot":"5f3c9a1e","provider":"yasb","topics":["daemon","focus"]}}
{"seq":1,"rev":57,"snapshot":{"daemon":{...},"focus":{...}}}
```

### The State in a Session

- A `snapshot` carries every followed topic in full. An `update` carries
  what changed since the session's previous message with state in it, in
  the shape of [Updates](#updates): a changed monitor in full, a removed
  one by its id.
- `seq` counts the messages that carry state, from 1, in each session. A
  gap means one was missed; ask for `resync`.
- `rev` is the state's revision since the daemon started, the same for
  every session. It only grows while `boot` stays the same.
- A slow bar is not sent every state in turn. Its writer sends the latest
  state the daemon published, as one update from what the bar was sent
  last, so a bar that falls behind catches up in one message, and the
  daemon keeps no queue of states for it.

### Requests

Every request carries an `id` the bar picks, and its answer carries it
back:

| Request | Answer |
| --- | --- |
| `{"id":1,"run":{"action":"goto-workspace-3","monitor":"m-41c6e2"}}` | `{"id":1,"ok":{"rev":59}}` |
| `{"id":2,"subscribe":{"topics":["monitors","focus"]}}` | A snapshot of those topics, with `"id":2`; they are what the session follows from then on |
| `{"id":3,"resync":{}}` | A snapshot of the topics followed, with `"id":3` |
| `{"id":4,"ping":{}}` | `{"id":4,"ok":{}}` |

- `run` carries an action out as a click on that monitor's bar does,
  named as in `bar.toml`'s click bindings. Without `monitor` it acts on
  the focused monitor. An action that acts on the focused window, such as
  `close-focused`, is refused: a bar acts on monitors and workspaces,
  and a program running in the background should not close or move
  whichever window the user happens to be in. `ok.rev` is the revision once the action was applied,
  and the session was sent a state at that revision or later before the
  answer.
- A session has at most 32 `run` requests waiting for the daemon; the
  bar's next ones wait until one is answered.

### Errors

An error answers a request by its `id`, or the session when there is
none:

```json
{"id":5,"error":{"code":"not-allowed","message":"`close-focused` acts on the focused window, which a bar may not"}}
```

| Code | Meaning |
| --- | --- |
| `busy` | The daemon serves as many sessions as it may; the session closes |
| `unsupported-api` | No version in common, with `api` listing the daemon's; the session closes |
| `malformed` | Not JSON, not shaped as its kind is, a request before `hello` (which closes), or a second `hello` |
| `too-large` | Over 1 MiB; the session closes |
| `unknown-request` | A kind of request the daemon does not know |
| `not-allowed` | An action that acts on the focused window |
| `no-monitor` | The monitor named is not connected |
| `unknown-action` | An action the daemon does not know |
| `failed` | The action was carried out, and something it needed failed |
| `stopping` | The daemon is stopping and carries out nothing more |

A bar reads a code it does not know as any other failure.

### Limits

`Limits::default()` holds what the daemon allows, the same on every
platform:

- `hello` must come within 5 seconds of connecting.
- A message must be written within 10 seconds; a bar that reads nothing
  for longer is dropped.
- 32 sessions, and 64 connections counting those still to send `hello`.
  Past 64 a connection is closed unanswered.
- 32 `run` requests waiting per session.

### Closing

When the daemon stops, every session is sent `{"bye":{"reason":"stopping"}}`
before the windows are put back, and the daemon waits at most a second
for the bars to read it before it goes on. A session that ends without
`bye` is a daemon that crashed or a pipe that broke. A bar closes its
session by closing its end.

### How It Is Built

The sessions are core's, so every platform serves them alike. A platform
provides the endpoint, through `ApiListener` and `ApiConnection`, which
splits into a reader and a writer neither of which waits for the other.
`serve_api()` accepts bars and reads each one's requests on a thread of
its own; a `SessionWriter` on another thread sends it the state and the
answers. They meet in the `Hub`: the daemon loop publishes each new state
to it and is never held up by a bar, and what a session asks of the loop
goes to it as a `LoopRequest`, whose `Reply` answers `stopping` if it is
dropped unanswered. The first session asks the loop to publish, and
nothing is sent until a state was published while a session was open.

The sessions are tested against an in-memory pipe in
`crates/mosaico-core/src/api/tests/`. `metrics_tests.rs` counts the
messages and bytes a bar is sent in each scenario, checked against
`metrics/api.toml` by `cargo xtask metrics`.

## The Command Line

A bar that reads a command's output, or runs a command on a click, uses
`mosaico query`, `mosaico subscribe` and `mosaico run` instead of the pipe.
They open a session like any other bar, not a CLI request, so they keep
working with a daemon of another release that speaks a version in common.
How to use them is in the guide's [CLI page](../website/src/guide/cli.md);
this is what they do.

- `query [TOPIC...]` prints the first snapshot as JSON and exits. Only the
  topics named are printed, though every session follows `daemon`.
- `run ACTION` sends `{"id":1,"run":...}` and exits once it is answered. It
  waits at most 10 seconds for each message.
- `subscribe` prints each message as the daemon sent it, one per line.
  - `--full` prints `{"rev":59,"state":{...}}`, the topics asked for with
    every update applied, on subscribing and whenever it changes, then
    every other message but `welcome`. The daemon still sends only
    differences; the command keeps the state.
  - `--requests` sends each line read on standard input as a request and
    prints the replies, whatever the output. A line read before the
    session's snapshot waits for it, and a `run` that names no monitor is
    then given `--monitor`'s. A line that is not JSON is answered
    `{"error":{"code":"malformed","message":"not JSON"}}`. With nothing read
    for 15 seconds the command ends, so a bar sends `{"id":N,"ping":{}}`
    every 5 seconds, and a bar that stops without ending the command loses
    its session.
  - `--wait` waits for a daemon that is not running, trying every half
    second, and connects again when it goes away, asking for the topics of
    the last `subscribe` it sent. On losing the daemon it prints
    `{"disconnected":{"reason":"stopping"}}` (the reason `bye` gave, or
    `lost`), except under `--field`, then the new `welcome` and snapshot.
    A request waiting for an answer, or sent while there is no daemon, is
    answered `{"id":N,"error":{"code":"disconnected","message":"..."}}`.
    `disconnected` is the command's own code; the daemon never sends it.
  - It ends on `bye` or a lost session (without `--wait`), when standard
    input ends and every request sent is answered (with `--requests`, so
    `echo '{"id":1,...}' | mosaico subscribe --requests` prints the
    answer), when a write to standard output fails,
    and when the program that started it exits, so a bar that dies takes
    its command with it.
- `--monitor ID` takes a monitor's id, or its index from the left when it
  is all digits. It leaves only that monitor in `monitors`, and in
  `removed.monitors` of each update, and `run` and `--requests` act on it.
  `run` turns an index into the id from `monitors`, and is refused with
  `no-monitor` when no monitor is at that index.
- `--field PATH` prints one value as a line of text: keys and list indexes
  joined by dots, such as `monitors.0.active_workspace`. A string is
  printed with backslashes, line feeds and carriage returns escaped as `\\`, `\n` and
  `\r`; a number or a boolean as written; a list or an object as compact
  JSON; a null or a missing value as an empty line.

| Exit | When |
| --- | --- |
| 0 | Done, or ended by `bye`, its input, its output or its parent |
| 1 | Refused, no daemon running, or the session lost (without `--wait`) |
| 2 | A usage error |
| 3 | No bar API version in common; it names the versions each side speaks |

A platform provides two things. `ApiClient` connects to the daemon's bar
API endpoint, as a reader and a writer that do not wait for each other, and
reports `NotRunning` when no daemon listens. `ParentWatch` names the
program that started the command and waits for it to exit; a platform that
cannot tell waits forever rather than end a command early. The client is
tested in core against the in-memory pipe and a real `Hub`.

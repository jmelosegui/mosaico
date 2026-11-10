# Phase 36: Bar Providers and the Bar API

**Status:** Pending

**Goal:** One contract that every bar follows, with the built-in bar as its
first implementer, and one setting that picks the bar: the built-in bar (the
default), another bar that follows mosaico through a public, versioned API, or
none. Whichever bar it picks shows what the built-in bar shows of mosaico,
follows every change without polling, runs what a click on the built-in bar
runs, and gets its space without offsets tuned by hand.

## Overview

Today the built-in bar is the only bar that can follow mosaico, and it reads
the tiler directly: `Tiler::bar_states()` and `Tiler::mark_maximized()` build
its `BarState`s after every action, event batch and tick. With
`enabled = false` in `bar.toml` it draws nothing and keeps no space, but the
bar manager is still built, `bar.toml` is still merged at every start and
reloaded on every change, the tick thread wakes the daemon every second, and
the bars' state is still built, walking the desktop for `maximized`, for no
bar. Another bar has nothing to follow: the only state a program can read is
`mosaico debug state`, a dump with no stability promise, by polling. A bar
that registers with the system as an app bar gets its space, because mosaico
tiles into the work area the system reports, but neither YASB nor Zebar does
that by default, and any other bar is tiled under.

This phase adds:

- **The contract.** `api::State`, a public model of what a bar shows of
  mosaico, and `BarProvider`, what the daemon asks of whichever bar is active:
  take the latest state, say how far it reaches from each monitor's edge, and
  hand back the actions its clicks ask for. The built-in bar is its first
  implementer, drawn from `api::State` and nothing else, so every other bar can
  show what it shows.
- **The setting.** `[bar] provider` in `config.toml`: `mosaico` (the default),
  `external` (whichever bar connects through the API), or `none`, with `zebar`
  and `yasb` once their integrations land. The settings app shows it as a
  dropdown at the top of the Bar page. Under any provider but `mosaico`, none
  of the built-in bar's code runs.
- **The API.** A pipe of its own with sessions that get the whole state, then
  numbered updates, and send actions and reservations. `mosaico query`,
  `mosaico subscribe` and `mosaico run` are thin clients of it, so any bar that
  can run a command follows mosaico with no library.
- **Space.** A reach from each monitor edge, held by the active provider and
  ended with it, combined with the system's work area by taking the further,
  so it never adds up with an app bar's.
- **Named providers.** Zebar and YASB integrations, a setup command each, and
  doctor checks.

Phase 35 is on `main`. A bar's command acts on its monitor through
`Origin::Bar` and never moves the pointer (35.1). Only actions whose
`Action::scope()` is `Monitor` or `Daemon` may come from a bar (35.2). Bars
have a `BarId` with a generation, and the CLI protocol is at version 2 (35.1).
The bar API has a version of its own, which starts at 1.

## Design Goals

- One contract for every bar. The built-in bar implements `BarProvider` and
  reads only `api::State`, so the API can never fall behind what the built-in
  bar shows; a test keeps it that way.
- One bar serves every monitor, picked by one setting. Without the setting,
  nothing changes.
- The choice is explicit. mosaico never guesses which bar to follow by looking
  for running programs, so two bars never fight over the same space.
- Under any provider but `mosaico`, none of the built-in bar's code runs: no
  bar windows, no reading, merging or watching of `bar.toml`, no refresh timer,
  no machine readings, and no space kept for it.
- The state is built only when something a bar shows changed, and only while
  something reads it: the built-in bar or a session. Nothing walks the desktop
  more often than the built-in bar does today.
- A bar never polls and never misses a change. It gets the whole state when it
  subscribes, then numbered updates, so applying them in order gives the
  daemon's state, and a missing one shows.
- A slow or stuck client never holds up the daemon, never makes it buffer
  without bound, never blocks another client, and is never more than one state
  behind. Nothing is written to a client from the daemon's loop.
- The API is versioned on its own, apart from the CLI protocol. Within a
  version, changes only add, clients ignore what they do not know, and example
  files pin the wire shape.
- Space belongs to the bar that holds it and ends with it. No reservation
  outlives its session, none adds up with what the system keeps, and none is
  keyed by a monitor index that changes when a display is plugged in.
- Only the user's own processes on this machine, at the user's integrity or
  above, can connect. A client gets window titles only when it asks for them.

## The Contract

### `api::State`

The public model of what a bar shows of mosaico. It is shaped as the wire is,
kept apart from the tiler's types, and built by one builder from the tiler and
the daemon's facts. Topics group it:

| Topic | Holds |
|-------|-------|
| `daemon` | mosaico's version, the API version, the instance, the provider, whether the hotkeys are paused, the update notice, the workspace mode and `wrap` |
| `theme` | the theme's name and flavor, and its named colors |
| `monitors` | each monitor's id, index, name, device, scale, bounds, the system's work area, the tiling area, whether a maximized or fullscreen window covers it, and its workspaces: number, name, layout, monocle, windows in layout order, the one focused last |
| `focus` | the focused monitor, workspace and window, with that window's title and program |

- A monitor's `id` in the API is a `StableMonitorId`, assigned by the tiler
  and kept in `MonitorState` next to the platform's `MonitorId`, which stays
  the platform's handle for lookups. It is taken from the monitor's key where
  the key is unique among the connected displays and disambiguated where it is
  not, parked with the monitor's workspaces when it disconnects, and restored
  with them by the same matching (`display_rebuild.rs`, `memory.unpark`), so
  a display that comes back on another port keeps its id. Two connected
  monitors never share one.
- `index` counts from the left, as `bar.toml`'s `monitors` does, and
  `monitors` is always in index order: a snapshot sends it sorted, an update
  that changes the order carries every monitor that moved, and a client sorts
  by `index` after applying one, as `--full` does. `name` is what the system
  calls the display, and `device` the name other programs use for it; a bar
  finds its own monitor by any of them, or by its bounds.
- Rects are in the desktop's pixels; `scale` is for a bar that works in scaled
  units.
- Workspaces are numbered from 1, as actions number them. `name` is the number
  today, so names the user sets later need no new field.
- The focused window's title is cut at 1,000 characters and reaches only the
  sessions that subscribe to `focus`. The integrations show it as text, never
  as markup, and nothing the API adds logs titles above debug level.
- Machine readings (CPU, memory, media, time) are not in the state: they are
  not mosaico's, and every bar reads them its own way.

### When the state is built

The state is rebuilt only when it is dirty and something reads it. It is
dirty when:

- the tiler says the bars changed (`take_bars_changed()`), or a workspace,
  layout, monocle, visibility or focus changes;
- the focused window's title changes (`TitleChanged` for that window is read
  at once; the scheduler already keeps the last one per window per batch), or
  focus moves to another window, whose title is read then;
- the hotkeys are paused or resumed, the update notice arrives, the config is
  reloaded, the theme changes, a display or work area changes, or a
  reservation changes.

`maximized` is worked out again only when the tiler says the bars changed, a
workspace or visibility change asks for it, the work area or the displays
change (coverage is measured against the work area, so it can change with no
window event), and on the first build, not on every rebuild. A batch of
`LocationChanged` alone, as during a drag or a transition, leaves the state
clean. With no session and the provider not `mosaico`, nothing builds the
state at all.

### `BarProvider`

What the daemon asks of the active bar, in `bar/provider.rs`:

- `show(&State)`: take the latest state. The provider draws it, or forwards
  it.
- `reach(&Monitors) -> Reach`: how far the bar reaches from each edge of each
  monitor, by stable id. `Monitors` is the tiler's own light list of each
  monitor's ids, index, bounds and scale, so space is worked out whether or
  not anything builds the state. Tiles keep clear of the reach, or of the
  system's work area, whichever reaches further.
- `take_runs() -> Vec<BarRun>`: the actions its clicks asked for since the
  last call, each with the monitor it was asked on. They run from
  `Origin::Bar` under the scope check of 35.2, whichever bar asked.
- `due() -> Option<Instant>`: when it next needs to wake the daemon. For the
  built-in bar that is the soonest of its clock, its machine readings (CPU,
  memory, media) and the wheel's quiet period (35.1); `None` when nothing
  waits. A wake for a reading redraws the built-in bar from the state it
  holds, without rebuilding `api::State`.
- `stop()`: release everything it holds, before the next provider starts.

The daemon holds one `Box<dyn BarProvider>` and talks to no bar any other way.
The API's sessions are open under every provider, for scripts as much as bars,
so their `run` requests reach the loop on their own, and run exactly as
`take_runs` does: the same origin, the same scope check, the same path. Three
implement it in this phase:

| Provider | `show` | `reach` | `take_runs` | `due` |
|----------|--------|---------|-------------|-------|
| `BuiltinBar` (`mosaico`) | `BarState::from_state` for each monitor it shows on, then `BarManager::update` | its height below the top of the system's work area, on the monitors `bar.toml` names | its clicks and wheel steps (35.1, 35.2) | the soonest of its clock, readings and wheel |
| `ApiBar` (`external`, `zebar`, `yasb`) | nothing; the hub publishes the same state to the sessions | `[bar.reserve]`, and the reservations of the sessions bound to it | nothing (see below) | `None` |
| `NoBar` (`none`) | nothing | nothing | nothing | `None` |

The built-in bar stays inside the daemon. A click is hit tested against what
it last drew, stale bars are told apart by their `BarId` generation, and the
click's action runs on the daemon's loop against the state the bar shows. In a
process of its own, every click would cross a process boundary and could act on
a state the bar has not shown, and the daemon would have a second program to
start, watch and restart. It follows the contract by construction instead:

- `BarState::from_state(&State, monitor id)` is a pure function, and the
  builder of `api::State` takes over what `Tiler::bar_states()` and
  `Tiler::mark_maximized()` work out today. Those two go.
- A test builds `api::State` from the tiler `Harness` across the existing bar
  scenarios, encodes it to JSON, decodes it, draws the built-in bar from both
  with a fake renderer, and expects the same content.
- `bar/` imports nothing from `tiler/` but the types `api::State` is made of,
  so a field the built-in bar needs has to be added to the API first.

## The Provider Setting

### Configuration

`config.toml`:

```toml
[bar]
# The bar that shows mosaico, on every monitor:
# "mosaico" (default): the built-in bar, styled in bar.toml.
# "external": whichever bar connects through the bar API.
# "zebar", "yasb": that bar, with mosaico's integration.
# "none": no bar, and no space kept for one.
provider = "mosaico"

# external, zebar and yasb: space for a bar that cannot reserve its own, from
# each edge of every monitor, in pixels at 100% scale, multiplied by each
# monitor's scale. Tiles keep clear of it, or of what the system keeps,
# whichever reaches further. It is kept whether the bar runs or not.
[bar.reserve]
top = 0
bottom = 0
left = 0
right = 0

# One monitor, by its index from the left, in place of the table above.
# [bar.reserve.monitor.1]
# top = 40
```

The provider lives in `config.toml`, not `bar.toml`: a setting that decides
whether `bar.toml` is read cannot be in `bar.toml`, and `config.toml` is always
read first.

- The value is compared without regard to case, as theme names are. A value
  mosaico does not know is read as `external`, so the built-in bar never draws
  over a bar the user picked. `mosaico start` prints the warning, the log has
  it, and doctor shows it.
- A `config.toml` that does not parse at startup today falls back to
  `Config::default()`, whose provider is `mosaico`. The loader reads the
  `[bar]` table on its own first; if even that cannot be read, the provider is
  `external` with no reservation, and `mosaico start` and doctor say why. A
  reload that fails keeps the old config, as today.
- `enabled = false` in `bar.toml` keeps meaning no bar and no space under
  `mosaico`, and doctor suggests `provider = "none"`, which says the same
  without reading `bar.toml`.
- `[bar.reserve]` is in scaled pixels so one value fits monitors of mixed DPI;
  the per-monitor tables cover the rest. It is static: it holds whether the bar
  runs or not, so a bar that can reserve through its session should.

| Provider | Built-in bar | `bar.toml` | Space mosaico keeps | Sessions that may reserve |
|----------|--------------|------------|---------------------|---------------------------|
| `mosaico` | On the monitors `bar.toml` names | Read, merged, watched | The bar's height below the top of the system's work area | none |
| `external` | Off | Not read | `[bar.reserve]`, and what sessions reserve | any |
| `zebar`, `yasb` | Off | Not read | `[bar.reserve]`, and what that bar's sessions reserve | those that say they are that bar |
| `none` | Off | Not read | Nothing | none |

The bar API is open under every provider, so scripts and widgets can follow
mosaico next to the built-in bar too; only who may reserve depends on the
provider.

### In the settings app

The Bar page gets a **Bar** dropdown at its top, before its tabs:
"Mosaico bar", "Another bar", "No bar", and "Zebar" and "YASB" once their
steps land. It is `[bar] provider`, described in `config_settings.rs` as a
`SettingKind::Choice`, so the window draws, checks, saves and searches it as it
does every other choice.

- Under "Mosaico bar", the tabs edit `bar.toml` as today.
- Under any other choice, the tabs give way to a note that `bar.toml` is not
  used while another bar is chosen, and, for every choice but "No bar", the
  `[bar.reserve]` values and, once 36.7 lands, the sessions connected and what
  each reserves.
- Saving the choice writes `config.toml`, and the daemon's reload switches
  bars live.

### Switching on a reload

A reload of `config.toml` that changes the provider is applied in one step, so
windows move once:

1. The new provider is made. Under `mosaico`, `bar.toml` is read and merged
   now, before any space is worked out.
2. The old provider's `stop()` runs: the built-in bar's windows, refresh and
   readings go, or the old provider's sessions are told (36.7).
3. The space is worked out for the new provider. Where the new provider has
   no reach yet on an edge the old one had (a bar that has not connected yet),
   the old reach is kept until the new provider's first reservation on that
   monitor, or 5 s, whichever comes first, so windows do not move for no bar
   and then again for the new one.
4. The tiler retiles only the monitors whose tiling area moved.
5. The new provider's `show` takes the current state, and every session gets
   an update with the new `daemon.provider`.

While the provider is not `mosaico`, the config watcher ignores `bar.toml`, and
the file is read fresh when the provider is `mosaico` again.

## Space for the Bar

A reach says how far a bar extends from an edge of a monitor. On each edge,
the tiling area keeps clear of whichever reaches further: the system's work
area or the bar.

- The system's work area already leaves out the taskbar and every bar that
  registers as an app bar, and mosaico already follows it. Such a bar needs
  nothing more.
- A bar that registers and also reserves is never counted twice, since both
  reach the same line. A bar drawn below a taskbar on the same edge reserves
  down to its own bottom. So a bar reserves without knowing what the system
  keeps.
- `Input::Bar(BarReservation)` (a height at the top) becomes `Input::Reach`,
  four edges per monitor by id, and the tiling area is the system's work area
  kept clear of them. It is worked out again on every work area change,
  display change, reload, reservation and session end, always from the
  system's work area, so nothing adds up, and only monitors whose tiling area
  moved are retiled.
- A reach past a quarter of the monitor on an edge is refused (`too-large`),
  so a sidebar fits on a portrait monitor and nothing can take a monitor away.
- `monitor_at_point` uses each monitor's full bounds, so a pointer on a bar at
  any edge, reserved or registered with the system, is on its monitor (it
  counts only the strip above the tiling area today).
- `Display` gains `bounds`, so a reach is measured from the monitor's edge,
  not from the work area.
- A window the user maximizes still covers a reserved strip, as it covers the
  built-in bar; only a bar that registers with the system keeps maximized
  windows out too. The guide says so for each named bar.
- `mosaico doctor` shows per monitor and edge what the system keeps and what
  mosaico keeps, and from which source.

## The Bar API

### Sessions

A bar opens a session on `\\.\pipe\mosaico.bar` (`\\.\pipe\mosaico-<name>.bar`
for a named instance; a dot cannot be part of an instance name, so it never
collides with an instance called `bar`). It is a pipe of its own because the
CLI's is one request and one reply per connection, from a CLI of the daemon's
own release, while a session lasts as long as its bar runs and was built
against some other release. The daemon owns the endpoint; a bar never creates
one for mosaico to connect to, so nothing is left on disk and nothing breaks
when the bar starts first.

Messages are one compact JSON value per line, at most `MAX_MESSAGE_BYTES`
(1 MiB), as the CLI's are:

```
bar    {"hello":{"api":[1],"bar":"zebar","client":"mosaico-zebar 1.0","topics":["daemon","theme","monitors","focus"]}}
daemon {"welcome":{"api":1,"mosaico":"<version>","boot":"5f3c9a1e","provider":"zebar","topics":["daemon","theme","monitors","focus"]}}
daemon {"seq":1,"rev":57,"snapshot":{"daemon":{...},"theme":{...},"monitors":[...],"focus":{...}}}
daemon {"seq":2,"rev":58,"update":{"monitors":[{...}],"focus":{...}}}
bar    {"id":1,"run":{"action":"goto-workspace-3","monitor":"m-41c6e2"}}
daemon {"seq":3,"rev":59,"update":{"monitors":[{...}],"focus":{...}}}
daemon {"id":1,"ok":{"rev":59}}
daemon {"bye":{"reason":"stopping"}}
```

- `hello` comes first, within 5 s, or the session is dropped. The daemon sends
  nothing before it, `busy` included.
- `api` lists the versions the bar speaks. The daemon picks the highest it
  speaks too, or answers `unsupported-api` with the versions it has, and
  closes.
- `bar` names the bar the session speaks for, which decides whether it may
  reserve (see Space through the API). It is optional; a session without it is
  `external`. `mosaico subscribe --bar NAME` sets it.
- `daemon` is always among a session's topics, whatever it asks for, so every
  session sees a provider switch, a pause and the update notice.
- Topics the daemon does not know are left out of `welcome`, not refused. A
  request the daemon does not know is answered `unknown-request`, and the
  session goes on; there is no capability list to go stale.
- `boot` names the daemon's start and changes when it restarts.
- Requests carry an `id` the bar picks, and their reply carries it back:
  `run`, `reserve`, `release`, `subscribe`, `resync` and `ping`.
- `bye` is sent to every session on `mosaico stop`, at shutdown and on an
  instance restart, so a bar can tell a stop from a crash. Every session gets
  it, however it connected.

### Snapshots and updates

- A snapshot holds every subscribed topic in full. `subscribe` replaces the
  session's topics and is answered with a snapshot of all of them, carrying
  the request's `id`; `resync` likewise.
- An update holds what changed since the session's previous message: `daemon`,
  `theme` and `focus` in full when they changed, each monitor that changed, in
  full, by `id`, and `removed: {"monitors": [ids]}`. Applying one is replacing
  objects by id; a bar never folds events into a model of its own.
- `seq` counts the session's messages that carry state, from 1, so a missing
  one shows, and a bar that sees a gap sends `resync`. `rev` is the state's
  revision since `boot`.
- Updates may skip states: a session that falls behind gets the difference to
  the latest state, not every state in between.
- A reply to `run`, `reserve` or `release` carries `rev`, the state's revision
  once the request was applied, and comes after the session has been sent a
  state at that `rev` or later, or at once when no subscribed topic changed.
  So when a bar gets the reply, what it shows already includes the request's
  effect, if it had one, and possibly later ones.

### Back-pressure

- The daemon loop publishes the latest state to the hub, an `Arc` swap, when
  it rebuilt one. It never writes to a client and never waits for one.
- Each session has a writer thread. When it can write, it works out the
  difference between the state it last sent and the latest one, by value, and
  sends it, so a bar that reads slowly gets fewer, larger updates and nothing
  piles up. A session's failure ends that session only.
- A write that does not finish within 10 s drops the session.
- At most 32 sessions are open, past which `hello` is answered `busy`. At most
  32 of a session's requests wait for the daemon; past that its next ones are
  not read until replies go out, so a client that floods only slows itself.

### Commands

- A bar runs an action with the monitor it acts on, by id, as a click on that
  monitor's built-in bar does: from `Origin::Bar` (35.1), so an action on a
  monitor gives that monitor focus first and nothing moves the pointer.
- Actions are named by `Action`'s `Display` form (`goto-workspace-3`,
  `cycle-layout`, `toggle-pause`, ...), the same strings `bar.toml`'s click
  bindings use. Only those whose scope is `Monitor` or `Daemon` (35.2) may run;
  the rest are answered `not-allowed`.
- They go through the daemon's action path, so they honour the workspace mode,
  and still run while the hotkeys are paused, as bar clicks do.

### Versions

- The API's version is a whole number, 1 to start with, apart from the CLI's
  `PROTOCOL_VERSION`. Within it, changes only add: fields, topics, requests
  and error codes.
- A bar ignores fields and messages it does not know. The daemon ignores
  `hello` fields it does not know.
- A change that would make a bar misread a message is a new version. A daemon
  keeps speaking the previous one for at least a year after the new one ships.
- Example files in `docs/bar-api/` pin the wire shape: a core test decodes
  each and encodes it back byte for byte. Changing one is an API change,
  reviewed as one. A JSON Schema is generated from the same types and checked
  against the examples.

## Space through the API

- A session reserves with `{"id":2,"reserve":{"monitor":"m-41c6e2","top":48}}`,
  or `"monitor":"all"`, in the desktop's pixels. Another one for the same
  monitor replaces it; `release`, or all zeros, ends it.
- Only sessions bound to the active provider may reserve: any session under
  `external`, and those whose `hello` said `"bar":"zebar"` (or `yasb`) under
  that provider. Others are answered `not-provider`. Two reservations on an
  edge are the same strip, and the further counts.
- A reservation lasts as long as its session. When a session ends without
  releasing, its space is kept 5 s more, so a bar that restarts does not make
  windows jump twice. `release`, a switch away and `mosaico stop` end it at
  once.
- At a switch, sessions no longer bound to the provider lose their
  reservations and see the new `daemon.provider` in their next update; the
  integrations hide their bar then, keeping their session, and show and
  reserve again when it is theirs.
- A reservation follows its monitor by id. One for a monitor that disconnects
  is kept until its session ends, and applies again when the monitor returns.
- Windows of the bar that holds a reservation are never tiled, so a bar needs
  no rule of its own and no program name is special-cased. The bar is the
  session's peer process, which the platform gives from the connection, or for
  `mosaico subscribe` the process that started it, which the CLI names in
  `hello` from the handle `ParentWatch` holds.
- A reservation reads the system's work areas again first, as `retile` does,
  since an app that sets the work area without saying so is not reported.

## The Command Line

`mosaico query`, `mosaico subscribe` and `mosaico run` speak the bar API, not
the CLI protocol, so they keep working with a daemon of another release that
speaks the version.

```
mosaico query [TOPIC...] [--monitor ID] [--field PATH]
mosaico subscribe [--bar NAME] [--topics LIST] [--monitor ID] [--full | --field PATH] [--wait] [--requests] [--reserve EDGE=PX]
mosaico run ACTION --monitor ID
```

- `--monitor` takes a monitor's id or index. It filters `monitors` to that one
  and sets the monitor `--requests` runs actions on when a request names none.
- `query` prints one snapshot and exits. With `--field`, such as
  `monitors.0.active_workspace` (index into the ordered list) or
  `focus.window.title`, it prints that value alone, as plain text, with
  newlines and backslashes escaped as `\n` and `\\`.
- `subscribe` prints the session's messages, one per line.
  - `--full` prints the whole state of the topics on subscribing and on every
    change; the CLI applies the updates, so a bar that keeps no state never
    polls, and the daemon still sends differences.
  - `--field PATH` prints that value, escaped, each time it changes.
  - `--requests` sends the requests it reads on standard input, one per line,
    and prints their replies as JSON lines whatever the output mode. It exits
    when no line comes for 15 s, so a bar sends `ping` every 5 s; a bar that
    closes a widget without ending its process loses its session and its
    space within 15 s.
  - `--wait` waits for a daemon that is not running and connects again when it
    restarts. On every new session it replays the last `subscribe` it
    forwarded and the reservations the daemon acknowledged and still holds
    (a reservation replaced, zeroed or released is replayed as it last
    stood, or not at all), fails the requests that were in flight, and prints
    a `{"disconnected":...}` line on losing the daemon, then the new `welcome`
    and snapshot, so a bar outlives a restart with no logic of its own.
  - `--reserve top=32` (or `bottom`, `left`, `right`, `@ID` for one monitor)
    holds that space for as long as the command runs.
  - It exits when its input ends (with `--requests`), when a write to its
    output fails, or when the process that started it exits.
- `run` runs one action on a monitor and exits, for bars that run a command on
  a click.
- Exit codes: 0 after `bye` (without `--wait`), 1 when no daemon runs (without
  `--wait`), 3 for a version mismatch, naming the versions each side speaks
  (2 stays clap's usage error).

## Security

The CLI pipe is created today with the default descriptor, which gives
Everyone and Anonymous read access, against `IpcServer`'s own contract (only
clients of the same user). Both pipes get:

- an explicit descriptor: the user and SYSTEM, network logons denied, and a
  mandatory label at the daemon's own integrity, so less trusted processes of
  the user can neither read nor write;
- `PIPE_REJECT_REMOTE_CLIENTS` on every instance and
  `FILE_FLAG_FIRST_PIPE_INSTANCE` on the first, so a pipe someone else created
  first is never joined;
- on the client side, `SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION`, so a
  server that is not the daemon cannot act as the user, and a check that the
  pipe's owner is the user and its label is at least medium, so an elevated
  CLI still reaches an unelevated daemon.

The boundary this keeps is other users, other machines and less trusted
processes of the user. Any process of the user at the same integrity can still
create an instance of the pipe; that is the user's own code, as with any
per-user IPC, and the plan claims nothing past it.

An elevated daemon: the bar pipe is labelled medium, so an unelevated bar
connects, and since it then sends requests into an elevated process, no bar
request runs a program, names a file, or acts on a focused window (35.2). The
CLI pipe keeps the daemon's integrity, so an unelevated CLI keeps whatever
access it has today to an elevated daemon. 36.1 confirms today's behaviour
with an elevated run and records it.

## Integrations

Each lives under `integrations/`, versioned with the API, and is described in
`docs/bar-api.md` and on the guide's new "Other Bars" page.

- **Zebar.** A widget pack of plain HTML and JavaScript, embedded in the
  binary, with one widget per monitor running
  `mosaico subscribe --bar zebar --wait --requests --full`. It finds its
  monitor by device name, draws workspaces, layout and the focused window in
  the `theme` topic's colors, runs clicks through the same process, reserves
  its strip, pings every 5 s, and kills its child on `pagehide`. While the
  provider is not `zebar` it hides its window and reserves nothing, keeping its
  session, and shows and reserves again when the provider is `zebar`, so a
  switch back from the settings app needs nothing started. Its allow-list
  admits the program setup writes, by exact path, with `argsRegex`
  `^subscribe --bar zebar --wait --requests --full$`.
  `mosaico bar setup zebar` writes it to `~/.glzr/zebar/mosaico/`, adds it to
  `startupConfigs` in `settings.json` (printing the entry instead if the file
  cannot be read), and runs `zebar start-widget-preset`, which Zebar forwards
  to its running instance.
- **YASB.** A `mosaico` service in YASB, one per YASB process, that holds one
  session with `"bar":"yasb"` and reserves the strip of every YASB bar window,
  and widgets for the workspaces, layout and focused window that read from it,
  so a widget never opens a session of its own. While the provider is not
  `yasb` the widgets hide and the service reserves nothing. It opens the pipe
  through Qt's local socket and finds a screen by the monitor's device name.
  The code follows YASB's contribution rules and goes to YASB as a pull
  request; the API's example files are its fixtures.
  `mosaico bar setup yasb` checks that the installed YASB has it and prints
  the entries for `config.yaml`, which mosaico does not edit. Until a YASB
  release has it, it prints YASB's own `windows_app_bar` setting and
  `[bar.reserve]` for space instead of a polling fallback.
- **Any bar that runs a command:** recipes for `mosaico subscribe --field`
  (a bar that reads lines), `--full` (one that reads JSON) and `mosaico run`
  for clicks, with `--reserve` or `[bar.reserve]` for space.
- `mosaico bar setup <bar>` writes files only. It switches the provider only
  when that bar is running, and otherwise prints how to start it and the line
  to set, so the built-in bar never disappears with nothing in its place.

## Migration and Compatibility

- A user who does nothing gets `provider = "mosaico"`: the same bar, on the
  same monitors, with the same space. An existing `config.toml` gains a comment
  block about `[bar]` (`merge_missing_config_sections`), with no value set.
- `enabled = false` in `bar.toml` still means no bar and no space.
- `mosaico debug state` stays a dump for people debugging mosaico;
  `mosaico query` is the API.
- The CLI protocol stays at 2. `Status` gains optional fields (the provider,
  the sessions, the space per monitor), which an older CLI ignores.
- The CLI pipe's new descriptor keeps out only other users, other machines and
  less trusted processes, none of which have reason to reach it.

## Edge Cases

1. A bar that starts before the daemon waits with `--wait`; the daemon never
   connects out to a bar, so there is no subscription to lose. When the daemon
   restarts, `bye` comes, and the next session shows a new `boot`, a fresh
   snapshot, and its reservation replayed.
2. A bar that crashes: its session ends, and its space 5 s later unless it
   reserves again first.
3. A provider switch during a drag or a transition (phases 34 and 35): the
   retile takes the usual path, and a window the user holds is left alone
   until it is dropped.
4. A display change: monitors keep their ids through the tiler's matching, so
   reservations and `--monitor` still mean the same display. A command for a
   monitor that just went is answered `no-monitor`.
5. Global workspace mode: a workspace command switches every monitor and gives
   focus to the one it names, as a click on the built-in bar does.
6. Hotkeys paused: commands still run, and `toggle-pause` from a bar resumes
   them.
7. A client that sends nothing, or a line over 1 MiB, is dropped; one that
   sends a malformed request gets `malformed`, and the session goes on.
8. Two bars both connected under `zebar`: both are bound to it, and the
   further reach on each edge counts. The guide says to run one.
9. `config.toml` and `bar.toml` saved together with a switch to `mosaico`: the
   switch reads `bar.toml` fresh, so the order of the reloads does not matter.
10. A bar killed while it was registered with the system as an app bar:
    Windows can keep its space out of the work area until Explorer works the
    areas out again. mosaico follows the work area either way, and the guide
    says how to get the space back.
11. Explorer restarts: the work areas change and mosaico follows them. A
    reservation through the API does not depend on Explorer, so it holds.
12. The update check finds a new version: it posts a message to the loop
    (instead of waiting for the tick that 36.4 removes), so the built-in bar
    and the `daemon` topic both update.

## Testing

Core tests run against fakes, with no pipe, process or desktop. Each step
lists its own tests. The `windows` crate tests the pipe with pipes of the
tests' own: many sessions, a read and a write at once, write deadlines, the
descriptor read back, a pipe created first under a low-integrity copy of the
test's token refused by the client, a remote-path open refused, and
`ParentWatch` holding the parent's handle from the start.
`tests/external_bar.rs` (on demand, on the desktop, with the user's daemon
stopped) runs a test daemon with `provider = "external"`, reserves through a
session, checks the tiling area and its return after the grace, then registers
a window of its own as an app bar inside a guard that removes it, and checks
that mosaico follows the work area. It joins the desktop suites in the testing
skill and the CI desktop job.

`FakePlatform` counts the queries the new code adds (`windows_on_screen`,
`facts`, title reads), so the metrics cover what the tiler reads as well as
what it asks to change.

## New and Modified Files

```
crates/
  mosaico-core/src/
    api/                         # NEW: State, its builder, diff and apply, the protocol, sessions, the hub, field paths
    bar/provider.rs              # NEW: BarProvider, BuiltinBar, ApiBar, NoBar
    bar/state.rs                 # BarState::from_state
    bar/refresh.rs               # NEW: the built-in bar's due time, in place of the tick
    config/bar_section.rs        # NEW: [bar] in config.toml, the provider check
    config/schema/config_settings.rs  # the provider and [bar.reserve] for the settings app
    config/template_config.rs, config/store/, config/reload/  # template, hint, bar.toml only under mosaico
    tiler/reach.rs               # NEW: Reach, how sources combine per edge
    tiler/input.rs, tiler/work_area.rs, tiler/empty_focus.rs, tiler/desktop_display.rs  # Input::Reach, Display.bounds
    tiler/bar_states.rs, tiler/manager_bars.rs  # go, into the api::State builder
    monitor.rs, tiler/display_rebuild.rs  # StableMonitorId, parked and restored
    daemon/scheduler.rs          # the provider's due time and the reservation grace
    system/parent_watch.rs       # NEW
    ipc/transport.rs, ipc/response.rs, instance.rs  # the security contract, Status fields, the API endpoint's name
  mosaico-windows/src/
    pipe_security.rs, api_pipe.rs, parent_watch.rs  # NEW
    ipc.rs, monitor.rs, monitor_identity.rs  # the CLI pipe's security; Display's new fields
    daemon_loop*.rs, daemon_threads.rs, daemon_types.rs  # the provider, the hub, Task::Api, no tick, the update message
  mosaico-settings/src/pages/bar/  # the dropdown and the note in place of the tabs
  mosaico/src/
    main.rs, platform.rs, action_commands.rs  # the new commands
    commands/query.rs, commands/subscribe.rs, commands/run.rs, commands/bar_setup.rs  # NEW
    commands/doctor_bar.rs, commands/context.rs, commands/test_support.rs  # doctor, an API client in Context and the fake World
  mosaico/tests/external_bar.rs  # NEW
integrations/zebar/, integrations/yasb/  # NEW
metrics/tiler.toml, metrics/api.toml     # the new scenarios and baseline
docs/
  bar-api.md, bar-api/           # NEW: the API for bar authors, its example files and schema
  status-bar.md, configuration.md, ipc.md, daemon.md, cli.md, monitor-management.md, testing docs
website/src/
  guide/other-bars.md            # NEW
  guide/status-bar.md, guide/configuration.md, guide/cli.md, SUMMARY.md
```

## Tasks

Every step runs fmt, clippy, the safe test suites and `cargo xtask metrics`,
blesses what it changes on purpose, and brings its docs, templates and config
hints, so it can merge on its own. 36.1 stands alone. 36.2 comes first of the
rest; 36.3 needs 36.2; 36.4 and 36.5 need 36.3; 36.6 needs 36.1 and 36.2; 36.7
needs 36.4 and 36.6; 36.8 and 36.9 need 36.7.

### 36.1 Pipe security

- [x] The security contract on `IpcServer` and the CLI client, in core, in
      behaviour terms
- [x] The explicit descriptor, remote rejection and first-instance flag for
      the CLI pipe; the client's identification level and owner and label
      checks
- [ ] Today's behaviour with an elevated daemon and an unelevated CLI,
      confirmed and recorded in `ipc.md`
- [x] Tests on pipes of the tests' own

### 36.2 The state model and the contract

- [x] `StableMonitorId`, assigned by the tiler, parked and restored with the
      monitor's workspaces
- [x] `api::State` and its builder, taking over `Tiler::bar_states()` and
      `mark_maximized()`; the dirty signal; the focused window's title read
      on `TitleChanged`
- [x] `Display` gains `bounds`, `name`, `device` and `scale`, in the contract
      and on Windows
- [x] `Reach` and `Input::Reach` in place of `BarReservation`, carrying only
      the top edge the built-in bar uses today
- [x] `BarProvider`, and `BuiltinBar` drawn through `BarState::from_state`;
      the daemon holds the provider and nothing else of the bar
- [x] `diff` and `apply`, with property tests: `apply(diff(a, b), a) == b`
- [x] The example files, their byte-for-byte test and the generated schema
- [x] The parity test: the built-in bar drawn from decoded JSON and from the
      built state, the same content
- [x] No visible change; every metrics scenario keeps its counts, now with the
      queries counted

### 36.3 The provider setting

- [ ] `[bar] provider` with `mosaico`, `external` and `none`, compared without
      case, an unknown value read as `external` and printed by `start`; the
      `[bar]` table read alone when the file does not parse
- [ ] `ApiBar` and `NoBar`; under them no bar manager, no `bar.toml` read,
      merged or watched, no machine readings, and the existing tick sends
      nothing to the loop until 36.5 removes it
- [ ] Switching on a reload in one step, in every direction, `bar.toml` read
      before the space is worked out
- [ ] The dropdown and the note on the settings app's Bar page
- [ ] doctor: the provider, `bar.toml` not read and why, settings the provider
      ignores
- [ ] The template, the hint for existing files, `configuration.md`,
      `status-bar.md` and the guide
- [ ] Metrics scenarios for the switches; what the built-in bar adds to the
      binary, measured with `cargo bloat` and recorded in the PR

### 36.4 Reach from four edges

- [ ] `Reach` on all four edges, combined with the system's work area by
      taking the further, only moved monitors retiled
- [ ] `[bar.reserve]` and its per-monitor tables, in scaled pixels
- [ ] The hand-over on a switch: the old reach kept until the new provider
      reserves, or 5 s
- [ ] `monitor_at_point` on each monitor's full bounds
- [ ] doctor: what the system and mosaico keep per monitor and edge

### 36.5 The refresh due time

- [ ] `BarProvider::due`, the soonest of the built-in bar's clock, readings
      and wheel quiet period, as a due time in the `Scheduler`, and the tick
      thread gone
- [ ] The update check posts a message to the loop

### 36.6 Bar API sessions

- [ ] The listener and connection contracts; the API pipe on Windows with
      overlapped handles and 36.1's security, labelled medium
- [ ] The hub, sessions, `hello`, `welcome`, topics, snapshots and updates,
      `subscribe`, `resync`, `ping` and `bye`
- [ ] Back-pressure: writer threads, the write deadline, the session and
      request limits
- [ ] `run`, from `Origin::Bar`, with the scope check, on the same path as
      `BarProvider::take_runs`, its reply after its update
- [ ] `docs/bar-api.md`, and the API in `ipc.md` and `daemon.md`
- [ ] Tests over in-memory connections, among them a writer that blocks while
      the daemon keeps publishing and another session keeps getting updates;
      metrics with a client, and `metrics/api.toml`

### 36.7 The commands and space through the API

- [ ] The client contract and its Windows pipe client
- [ ] `mosaico query`, `mosaico subscribe` (with `--wait` replaying, the
      `--requests` keepalive, `--reserve`) and `mosaico run`
- [ ] `ParentWatch` and its Windows implementation
- [ ] `reserve` and `release`, bound to the provider by `hello`'s `bar`, the
      grace, the refusals, monitors followed by id, the provider's sessions told
      at a switch
- [ ] Windows of a reserving session's process left untiled
- [ ] doctor: the sessions, their bars and what each reserves; the settings
      app's list of them
- [ ] `cli.md` and the guide's CLI page; tests with the fake `World`
- [ ] The desktop suite `tests/external_bar.rs`

### 36.8 Zebar

- [ ] `zebar` as a provider and a choice in the dropdown
- [ ] The widget pack in `integrations/zebar/`, embedded
- [ ] `mosaico bar setup zebar`: the pack, `startupConfigs`, starting the
      widget, switching only when Zebar runs
- [ ] doctor: installed, running, the pack written and current, connected,
      reserving
- [ ] Tests for setup against a fake file system

### 36.9 YASB

- [ ] `yasb` as a provider and a choice in the dropdown
- [ ] The service and widgets in `integrations/yasb/`, offered to YASB
      upstream
- [ ] `mosaico bar setup yasb`: the check and the entries it prints
- [ ] doctor: installed, running, a release with the integration, connected,
      reserving
- [ ] The guide's "Other Bars" page, with the recipes for any bar

### Verification

- [ ] Manual test: switch providers live from the settings app, `mosaico` to
      `zebar` to `none` to `mosaico`, and see windows move once each time and
      nothing left behind
- [ ] Manual test: Zebar and YASB on every monitor, in both workspace modes,
      following switches, layouts, monocle and focus, and clicks on each acting
      on its own monitor without moving the pointer
- [ ] Manual test: a bar started before the daemon, the daemon restarted under
      it, Explorer restarted, a bar killed, and a Zebar widget closed while
      Zebar keeps running
- [ ] Manual test: a client suspended mid-session never slows the hotkeys or
      the built-in bar, and gets one update once resumed
- [ ] Run `tests/external_bar.rs` on the desktop
- [ ] Update `.plans/plan.md`

## Later

- `launch`: starting, restarting and stopping a named bar with the daemon.
  It needs to watch the bar's own process, not a launcher that exits at once,
  never end a program that also runs the user's other widgets, restart only
  after an abnormal exit, stop off the loop thread, and refuse to run while the
  daemon is elevated; the program names and folders belong to the platform.
- Icons: an `icon` request by key, answered with a PNG.
- A `windows` topic and a `focus-window` action, so a bar can list windows and
  focus one on a click, as a taskbar does. It needs each window's process id
  from the platform.
- A `system` topic with the readings the built-in bar takes, read only while a
  session subscribes.
- A local WebSocket for a web bar that can neither run a program nor open a
  pipe, with pairing designed for it (a token the user pastes, rotated by a
  command) when such a bar turns up.
- Publishing the Zebar pack to Zebar's marketplace.
- Names for workspaces in the config; the API has the field already.
- The built-in bar as a process of its own on the API, if crash isolation is
  ever worth its cost.
- The same contracts on a socket, for `mosaico-linux` and `mosaico-macos`.

## Open Questions

For the owner to decide. Each carries the plan's proposal.

1. A provider mosaico does not know: read as `external` (proposed), or as
   `mosaico`.
2. YASB: reserve through the API (proposed, consistent with every other bar),
   or turn on YASB's own `windows_app_bar`, which keeps maximized windows off
   it and re-registers after Explorer restarts. Zebar's registration follows
   no Explorer restart, so the API stays its default either way.
3. The endpoint's name: fixed, `mosaico.bar` (proposed). The daemon's instance
   lock is already a fixed global name, so a per-user name gains nothing.
4. Reservations through the API under `mosaico`, for a dock at another edge:
   refused (proposed, one bar at a time), or taken.
5. The numbers: 5 s of grace, 10 s write deadline, 15 s keepalive, 32 sessions,
   32 waiting requests, a quarter of a monitor per reach, titles cut at 1,000
   characters.
6. A Cargo feature to build without the built-in bar: no (proposed), unless
   36.3's measurement shows it is worth a second build.
7. The overlap after an API version change: a year (proposed), or a number of
   releases.

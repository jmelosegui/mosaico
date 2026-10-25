# Phase 36: Bar Providers and the Bar API

**Status:** Pending

**Goal:** Let one setting pick the bar that shows mosaico on every monitor:
the built-in bar, YASB, Zebar, any other bar, or none. Whichever it picks gets
what the built-in bar shows of mosaico through a public, versioned bar API,
follows every change without polling, can run what a click on the built-in bar
runs, and gets its space without offsets tuned by hand.

## Overview

Today the built-in bar is the only bar that can follow mosaico. With
`enabled = false` in `bar.toml` it draws nothing and keeps no space, but the
bar manager is still built, `bar.toml` is still merged at every start and
reloaded on every change, a tick thread wakes the daemon every second, and the
bars' state is built after every action and batch of window events, for no
bar. Another bar has nothing to follow: the only state a program can read is
`mosaico debug state`, a dump for people with no stability promise, by
polling. A bar that registers with the system as an app bar gets its space,
because mosaico tiles into the work area the system reports, but neither YASB
nor Zebar does that by default, and any other bar is tiled under.

This phase adds `[bar] provider` in `config.toml`, which picks the one bar for
every monitor; a state model in core, `api::State`, which the built-in bar is
drawn from too; the bar API, sessions that send the whole state when a bar
subscribes, then numbered updates, and take a bar's actions and requests for
space; `mosaico query` and `mosaico subscribe`, so any bar that can run a
command and read lines follows mosaico; space measured from the monitor's
edge, so it never adds up with what the system keeps; named providers for YASB
and Zebar, with integrations, an opt-in start and stop with the daemon, a setup
command and doctor checks; and an optional local WebSocket, off by default.

It builds on phase 35, and starts once that is on `main`. A bar's command acts
on its monitor through `Origin::Bar` and never moves the pointer (35.1). Only
actions whose `Action::scope()` is `Monitor` or `Daemon` may come from a bar
(35.2). Bars have a `BarId` with a generation (35.1), and the CLI protocol is
at version 2 (35.1). The bar API has a version of its own, which starts at 1.

## Design Goals

- One bar serves every monitor, picked by one setting. Without the setting,
  nothing changes. The built-in bar's `monitors` still says which monitors it
  shows on, and no other bar is mixed in on the rest.
- Under any provider but `mosaico`, none of the built-in bar's code runs: no
  bar windows, no reading, merging or reloading of `bar.toml`, no refresh
  timer, no machine readings, and no space kept for it.
- Every bar reads the same state. The built-in bar is drawn from the `State`
  the API sends, through a pure function, so any other bar can show what it
  shows of mosaico and run what its clicks run.
- A bar never polls and never misses a change. It gets the whole state when it
  subscribes, then numbered updates taken from the same sequence of states, so
  applying them in order always gives the daemon's state, and a missing one
  shows.
- A slow or stuck client never holds up the daemon, never makes it buffer
  without bound, and is never more than one state behind: when it reads again,
  it gets the difference between what it last got and the latest state.
- The API is versioned on its own, apart from the CLI protocol. Within a
  version, changes only add, clients ignore what they do not know, and tested
  examples pin the wire shape. A bar written for version 1 works with every
  daemon that speaks 1.
- A named provider changes defaults and lifecycle only. The API sends every
  client the same.
- No space is tuned by hand to make up for something else: a bar says how far
  it reaches, tiles keep clear of that or of what the system keeps, whichever
  reaches further, and no space is kept for a bar that is gone.
- Only processes of the same user on the same machine can connect. A client
  learns nothing the user's processes cannot read already, and gets window
  titles only when it asks for them.
- The tiler pays nothing for clients it does not have: every metrics scenario
  keeps its counts with no client connected.

## What Other Designs Teach

- An existing design with its bar in a separate process sends every
  subscriber the whole state with every event, so a bar never drifts. But it
  serializes the whole desktop on every command and event, subscribers or
  not, under the window manager's lock and with no deadline on the write, and
  one bar that read too slowly hung the window manager. Third-party bars
  create the endpoint and run the CLI to be connected to it, then run it again
  for the state and for every command, which was slow and set off antivirus
  scans.
  Subscriptions are lost on every restart, nothing has a version, and new
  releases broke bars, at times silently. Space comes from offsets the bar
  sends, which a more specific offset replaces, which nothing takes back when
  the bar exits, and which add to an app bar's.
- i3 and sway send typed events once a client subscribes, but no state, and
  warn that a query and an event can cross; i3bar asks for the workspaces
  again on every workspace event. i3 once froze when a client stopped reading,
  and now buffers for 10 s, without a limit, then drops it.
- Hyprland writes events as lines of text that any tool can read, and adds a
  `v2` event each time one needs more, since positional text cannot grow. A
  comma in a name makes a line ambiguous.
- niri sends the complete state first, then updates to it, so a client never
  queries. It numbers nothing, drops a client 64 events behind, and stops
  reading requests on a stream, so a client that sends commands needs a second
  socket.
- GlazeWM serves a WebSocket on a fixed local port, with a command that runs
  programs, and checks neither the origin nor a token, so a web page can reach
  it. Subscriptions start with no state, and its client matches replies by
  their text.
- yabai runs a shell command per event, and bars built on it start processes
  per event and query again.
- Wayland's ext-workspace ends each batch of changes with `done`, so a client
  never draws half a change, and says per object which requests it takes.

So this plan sends the whole state first and numbered differences after it,
one consistent change per message, on one connection that also takes requests
with ids. A slow client gets the difference from what it last got, so nothing
queues. The state is a public model of its own, apart from the tiler's types,
with additive versions, a two-way hello and examples that tests pin. The CLI
is a thin client of the same sessions, not the way in. Space is a reach from
the monitor's edge, held by a session and ended with it. And no port a web page
can reach is open unless the user opens one.

## The Provider

### Configuration

`config.toml`:

```toml
[bar]
# The bar that shows mosaico, on every monitor:
# "mosaico" (default): the built-in bar, styled in bar.toml.
# "yasb", "zebar": that bar, through the bar API, with mosaico's widgets.
# "external": any other bar that uses the bar API.
# "none": no bar, and no space kept for one.
provider = "mosaico"

# yasb and zebar: start the bar with the daemon, start it again if it stops,
# and stop it with the daemon. Turn the bar's own autostart off then.
launch = false
# yasb and zebar: the bar's program, when it is not where its installer puts it.
program = ""

# yasb, zebar and external: how far a bar that keeps no space itself reaches
# from each edge of every monitor, in pixels. Tiles keep clear of it, or of
# what the system keeps, whichever reaches further.
[bar.reserve]
top = 0
bottom = 0
left = 0
right = 0

# One monitor, by its index from the left as bar.toml's `monitors` counts it,
# or by the id `mosaico query monitors` prints, in place of the table above.
# [bar.reserve.monitor.1]
# top = 40
```

The provider lives in `config.toml`, not `bar.toml`:

- A setting that decides whether `bar.toml` is read cannot be in `bar.toml`.
- `config.toml` is always read, first, and its reload already starts and stops
  parts of the daemon (focus follows mouse). Switching the bar is one more.
- `bar.toml` stays the built-in bar's look and widgets, and `mosaico init`
  still writes it, so switching back to `mosaico` finds it.

The provider is read as `[theme] name` is: a value mosaico does not know is
reported, by `mosaico doctor` and in the log, and read as `external`, so the
built-in bar never draws over a bar the user picked, and the rest of the file
still loads. `enabled = false` in `bar.toml` keeps meaning no bar and no space
under `mosaico`, and doctor suggests `provider = "none"`, which says the same
without reading `bar.toml`. Sizes are in the desktop's pixels, as the tiler
counts them and as the built-in bar's `height` is.

| Provider | Built-in bar | `bar.toml` | Space mosaico keeps | Started by mosaico |
|----------|--------------|------------|---------------------|--------------------|
| `mosaico` | On the monitors `bar.toml` names | Read, merged, reloaded | The bar's height below the top of the system's work area | -- |
| `yasb` | Off | Not read | What the mosaico widgets reserve, and `[bar.reserve]` | With `launch = true` |
| `zebar` | Off | Not read | What the mosaico widget reserves, and `[bar.reserve]` | With `launch = true` |
| `external` | Off | Not read | `[bar.reserve]`, and what the bar reserves | Never |
| `none` | Off | Not read | Nothing; reservations are refused | Never |

One provider serves every monitor. The built-in bar on some monitors and
another bar on the rest is not a setup mosaico supports; the built-in bar's
`monitors` only says where the built-in bar shows. The bar API is open under
every provider, so scripts and widgets can follow mosaico next to the built-in
bar too; only the space depends on the provider.

### What a named provider buys

The API is the same for every provider. A named one adds what fits its bar.

| | `yasb` | `zebar` | `external` |
|---|--------|---------|------------|
| Space | the mosaico widgets reserve their bar's strip | the mosaico widget reserves its strip | `[bar.reserve]`, or what the bar reserves |
| With `launch = true` | `yasbc start`, and `yasbc stop` with the daemon | `zebar start-widget-preset`, and Zebar closed with the daemon when mosaico started it | -- |
| Setup | `mosaico bar setup yasb` | `mosaico bar setup zebar` writes the widget pack | the guide |
| doctor | installed, running, a release with the widgets, connected, space | installed, running, pack written and current, connected, space | connected, space |

Both bars can register with the system as app bars, and both have that off by
default (`windows_app_bar` in YASB, `dockToEdge` in Zebar). The integrations
reserve through their session instead. The space then ends with the bar, holds
when Explorer restarts, which Zebar's registration does not follow, and never
adds up with a registration the user turns on, since the further one counts.

**YASB.** YASB's widgets are Python inside YASB, and its custom widget runs a
command and waits for it to exit, so it cannot follow a stream. The
integration is a set of `mosaico` widgets offered to YASB upstream, like the
ones it has for other window managers: workspaces, layout and the focused
window. They open
a session on the API pipe through Qt's local socket, which is a named pipe on
Windows, find their screen by the monitor's session id (the native handle YASB
already matches window managers' monitors by), run actions on a click from that
monitor, and reserve their bar's strip. `mosaico bar setup yasb` checks that
the installed YASB has them and prints the entries for its `config.yaml`, which
mosaico does not edit. Until a YASB release has them, it prints a fallback: a
custom widget that runs `mosaico query --field` on a timer, at the cost of a
process per tick.

**Zebar.** Zebar's widgets are web pages, and its `shellSpawn` runs a program,
streams its output and writes to it, under an allow-list in the widget pack.
So the integration is a widget pack mosaico ships: one widget per monitor that
runs `mosaico subscribe --wait --requests`, applies the updates, finds its
monitor by device name (Zebar's `name`), runs actions on a click through the
same process, and reserves its strip. `mosaico bar setup zebar` writes it to
`~/.glzr/zebar/mosaico/`, with an allow-list that admits that command alone,
and it is published to Zebar's marketplace too.

Other bars were weighed. Rainmeter registers no app bar and cannot read a
stream, so it gets a recipe under `external`. Seelen UI is a whole shell with a
tiler of its own. Neither earns a name.

### Lifecycle

`launch` is off by default: both bars can start at login on their own, Zebar
may run widgets of the user's that mosaico must not stop, and starting another
program is the user's choice. With it on:

- Find: `program` first, then `PATH`, then where the bar's installer puts it.
  A bar that is not found is logged, and doctor says where mosaico looked.
- Start: once the API endpoint listens, so the bar's first session finds it. A
  bar already running is left alone, and mosaico never stops it.
- Restart: a bar mosaico started that exits is started again after 1, 2 and
  4 s. After three restarts within 2 minutes, mosaico gives up until the next
  reload or start, and warns.
- Stop: at `mosaico stop`, at shutdown and on a switch away, only a bar
  mosaico started: with its own stop command when it has one (YASB), or by
  asking its windows to close (Zebar), then ending the process after 3 s.

### Setup

- `mosaico bar setup <provider>` writes what that bar needs and sets
  `[bar] provider` in `config.toml`, so one command switches bars live.
  `--no-switch` only writes the files.
- What it writes is stamped with the API version and mosaico's release.
  Running it again replaces the files, keeping a copy of any the user changed,
  and doctor says when they are older than the running mosaico.
- `mosaico init` asks nothing new: the built-in bar stays the default.

### Switching on a reload

A reload of `config.toml` that changes the provider is applied in one step, so
windows move once:

1. The old provider goes. The built-in bar's windows are dropped, with its
   refresh timer and readings. A bar mosaico started is stopped; one the user
   started is left running, and the log and doctor say it now shows next to
   the new one.
2. The reservations are worked out for the new provider, those of the old
   provider's sessions ending at once, and the tiler retiles the monitors whose
   tiling area moved.
3. The new provider comes. Under `mosaico`, `bar.toml` is read, merged and
   drawn. With `launch`, a named bar is started.
4. Every session gets an update with the new `daemon.provider`.

While the provider is not `mosaico`, the config watcher does not read
`bar.toml` at all, and the file is read fresh when the provider is `mosaico`
again.

## The Bar API

### Sessions

A bar opens a session on the API endpoint: `\\.\pipe\mosaico.bar` for the
default instance, `\\.\pipe\mosaico-<name>.bar` for a named one. A dot cannot
be part of an instance name, so the name never collides with the CLI pipe of
an instance called `bar`.

It is an endpoint of its own because the two do different jobs. The CLI
protocol is one request and one reply per connection, answered one connection
at a time, from a CLI of the daemon's own release. A session lasts as long as
its bar runs, many are open at once, and the bar was built against some other
release.

Messages are framed as the CLI's are, one compact JSON value per line, at most
`MAX_MESSAGE_BYTES` (1 MiB):

```
bar    {"hello":{"api":[1],"client":"zebar-mosaico 1.0","topics":["daemon","theme","monitors","focus"],"capabilities":["commands","reserve"]}}
daemon {"welcome":{"api":1,"mosaico":"0.11.0","boot":"5f3c9a1e","provider":"zebar","topics":["daemon","theme","monitors","focus"],"capabilities":["commands","reserve"]}}
daemon {"seq":1,"rev":57,"snapshot":{"daemon":{...},"theme":{...},"monitors":[...],"focus":{...}}}
daemon {"seq":2,"rev":58,"base":57,"update":{"monitors":[{...}],"focus":{...}}}
bar    {"id":1,"run":{"action":"goto-workspace-3","monitor":"m-41c6e2"}}
daemon {"seq":3,"rev":59,"base":58,"update":{"monitors":[{...}],"focus":{...}}}
daemon {"id":1,"ok":{"rev":59}}
daemon {"bye":{"reason":"stopping"}}
```

- `hello` comes first, within 5 s, or the session is dropped. The daemon sends
  nothing before it, so a process that could only open the endpoint for
  reading learns nothing.
- `api` lists the versions the bar speaks. The daemon picks the highest it
  speaks too, or answers `{"error":{"code":"unsupported-api","supported":[1]}}`
  and closes.
- Topics and capabilities the daemon does not know are left out of `welcome`,
  not refused, so a bar can ask for what a newer daemon has and see what it
  got.
- `boot` names the daemon's start, and changes when it restarts.
- A request carries an `id` the bar picks, and its reply carries it back. The
  requests are `run` (see Commands), `reserve` and `release` (see Space for
  the Bar), `subscribe`, which changes the topics and is answered with a
  snapshot of them, `resync`, which asks for one, `icon` (36.6) and `ping`.
- `bye` says the daemon is stopping, so a bar can tell a stop from a crash.

### Topics

| Topic | Holds | Changes on |
|-------|-------|------------|
| `daemon` | mosaico's version, the API version, the instance, the provider, whether the hotkeys are paused, the update notice, the workspace mode and `wrap` | a pause, an update found, a reload |
| `theme` | the theme's name and flavor, the colors the built-in bar would use, the border colors, and the theme's named colors | a theme or color change |
| `monitors` | each monitor's id, session id, index, name, device, scale, bounds, the system's work area, the space mosaico keeps, the tiling area, whether a maximized or fullscreen window covers it, and its workspaces: number, name, layout, monocle, windows in layout order, the one focused last | a workspace switch, a layout or monocle change, windows coming, going or moving, a display or work area change |
| `focus` | the focused monitor, workspace and window, with that window's title, program and icon key | focus, the focused window's title |
| `windows` | every managed window: id, title, program, path, process id, class and icon key | windows coming and going, titles |

A snapshot of one monitor, pretty printed:

```json
{"seq": 1, "rev": 57, "snapshot": {
  "daemon": {"mosaico": "0.11.0", "api": 1, "instance": null, "provider": "zebar",
             "paused": false, "update": null, "workspace_mode": "per-monitor", "wrap": false},
  "monitors": [{
    "id": "m-41c6e2", "session_id": "0x10001", "index": 0, "primary": true,
    "name": "DELL U2720Q", "device": "\\\\.\\DISPLAY1", "scale": 1.5,
    "bounds": {"x": 0, "y": 0, "width": 3840, "height": 2160},
    "work_area": {"x": 0, "y": 0, "width": 3840, "height": 2112},
    "reserved": {"top": 48, "bottom": 0, "left": 0, "right": 0},
    "tiling_area": {"x": 0, "y": 48, "width": 3840, "height": 2064},
    "maximized": false, "active_workspace": 1,
    "workspaces": [
      {"number": 1, "name": "1", "layout": "bsp", "monocle": false,
       "windows": ["0x4014E", "0x5012A"], "focused": "0x4014E"},
      {"number": 2, "name": "2", "layout": "bsp", "monocle": false,
       "windows": [], "focused": null}]}],
  "focus": {"monitor": "m-41c6e2", "workspace": 1,
            "window": {"id": "0x4014E", "title": "notes.md - Notepad",
                       "program": "notepad.exe", "icon": "9c1f02"}}}}
```

- A monitor's `id` is derived from its `MonitorKey`, so it stays the same
  across reconnects when the display can be identified, and lasts until it
  disconnects otherwise. `session_id` is the platform's id until the next
  display change, `index` counts from the left as `bar.toml`'s `monitors`
  does, `name` is what the system calls the display, and `device` the name
  other programs use for it: a bar finds its own monitor by any of them, or by
  its bounds. Rects are in the desktop's pixels, and `scale` is for a bar that
  works in scaled units.
- Workspaces are numbered from 1, as actions number them. Their `name` is the
  number today, so names the user sets later need no new field.
- Titles reach only the sessions that subscribe to `focus` (the focused
  window's) or `windows`, cut at 1,000 characters. Any web page can set a
  window's title, so the integrations show titles as text, never as markup,
  and the daemon logs them at debug level only.
- An icon is asked for by the key the state gave, never by a path.

### Snapshots and updates

- A snapshot holds every subscribed topic in full.
- An update holds what changed since the session's previous message: `daemon`,
  `theme` and `focus` in full when they changed, every monitor and window that
  changed, in full, keyed by `id`, and `removed`, the ids of the monitors and
  windows that went. Applying one is replacing objects by id; a bar never
  folds events into a model of its own.
- `seq` counts the session's snapshots and updates from 1, so a missing one
  shows. A bar that sees a gap, or cannot apply an update, sends `resync`.
- `rev` is the state's revision since the daemon started, and `base` the `rev`
  of the session's previous snapshot or update, which the update applies to.
- The reply to `run` comes after the update that shows what the action did,
  and carries its `rev`, so a bar knows when its click took effect.

### Back-pressure

- The daemon loop publishes the state once per wake, when something changed:
  it builds the state once, sharing every monitor and window that did not
  change with the previous state, and wakes the sessions. It never writes to a
  client and never waits for one.
- Each session has a thread that writes to its client. When it can write, it
  sends the difference between the state it last sent and the latest one, so a
  bar that reads slowly gets fewer, larger updates, and nothing piles up. The
  difference and its JSON are worked out on that thread, not the daemon's.
- A write that does not finish within 10 s drops the session.
- At most 16 sessions are open; more are refused with `busy`. At most 32 of a
  session's requests wait for the daemon; past that its next ones are not read
  until replies go out, so a client that floods only slows itself.

### Commands

- A bar runs an action with the monitor it acts on, by id. It runs as a click
  on that monitor's built-in bar does, from `Origin::Bar` (35.1): an action on
  a monitor gives that monitor focus first, and nothing it does moves the
  pointer.
- Only actions whose scope is `Monitor` or `Daemon` (35.2) may be run; the rest
  are answered `not-allowed`, since an action on the focused window would act
  on another monitor's window for every bar but one.
- Commands go through the daemon's action path, with the hotkeys, the CLI and
  the built-in bar's clicks, so they honour the workspace mode, and still run
  while the hotkeys are paused, as bar clicks do.
- `mosaico action <action> --monitor <id or index>` runs one through the API,
  for bars that run a command on a click.

### Versions

- The API's version is a whole number, 1 to start with, apart from the CLI's
  `PROTOCOL_VERSION`. Within it, changes only add: fields, topics,
  capabilities, requests and error codes.
- A bar ignores fields and messages it does not know. The daemon ignores
  `hello` fields it does not know, and answers a request it does not know with
  `unknown-request` without closing the session.
- A change that would make a bar misread a message is a new version. A daemon
  keeps speaking the previous one for at least two releases after the new one
  ships.
- Example files in `docs/bar-api/` pin the wire shape: a core test decodes each
  and encodes it back byte for byte. Changing one is an API change, reviewed as
  one.
- `mosaico query` and `mosaico subscribe` speak the bar API, not the CLI
  protocol, so they keep working with a daemon of another release that speaks
  the version. Every other CLI command still needs the daemon's
  `PROTOCOL_VERSION`.

## Transports

| Transport | For | How it stays the user's |
|-----------|-----|-------------------------|
| The API pipe | bars with code of their own: YASB widgets, scripts, a native provider | same user only, no remote or low-integrity clients, the name taken by the daemon first |
| `mosaico subscribe` and `mosaico query` | any bar that can run a command and read lines | the CLI is a pipe client like any other |
| A local WebSocket, off by default (36.9) | web bars that can neither run a program nor open a pipe | loopback only, a token in the user's data folder, an origin allow-list |

### The command line

```
mosaico query [TOPIC...] [--field PATH]
mosaico subscribe [--topics LIST] [--monitor ID] [--full | --field PATH] [--wait] [--requests] [--reserve EDGE=PX[@ID]]
mosaico action ACTION --monitor ID
```

- `query` prints one snapshot of the topics asked for, all of them by default,
  and exits. With `--field`, such as `monitors.0.active_workspace` or
  `focus.window.title`, it prints that value alone, as plain text, for a bar
  that runs a command on a timer.
- `subscribe` prints the session's messages, one per line, as they come.
  - `--full` prints the whole state of the topics instead, on subscribing and
    on every change. The CLI applies the updates itself, so a bar that cannot
    keep state still never polls, and the daemon still sends differences.
  - `--field PATH` prints that value, as plain text, each time it changes.
  - `--wait` waits for a daemon that is not running, and connects again when
    it restarts, with a fresh snapshot, so a bar that starts first or outlives
    a restart needs no logic of its own.
  - `--requests` sends the requests it reads on standard input, one per line,
    so a bar that writes to the program it runs gets commands and reservations
    without a pipe of its own. Without it, standard input is not read, since
    some bars give a program none.
  - `--reserve top=32` (or `bottom`, `left`, `right`, with `@ID` for one
    monitor) holds that space for as long as the command runs.
  - It exits when its input ends (with `--requests`), when a write to its
    output fails, or when the program that started it exits, so a bar that
    dies takes its session, and its space, with it.
- Exit codes: 0 after `bye`, 1 when no daemon runs (without `--wait`), and 2
  for a version mismatch, which names the versions each side speaks.

### The WebSocket

A WebSocket only serves a web bar that can neither run a program nor reach a
pipe from a native side of its own: Zebar can run one, YASB can open the pipe,
and a Tauri or Electron bar can open it natively and pass the state to its
pages, with no port at all. A local port, on the other hand, is a door any web
page can knock on. Browsers apply neither the same-origin policy nor CORS to a
WebSocket handshake, local servers of desktop apps and dev tools have been
driven from web pages that way, and GlazeWM's checks nothing today. So it is
the last step, optional (36.9), off unless `[bar.websocket] enabled = true`,
and kept safe this way:

- It listens on 127.0.0.1 only, and clients use that literal address, since
  `localhost` may resolve to `::1`, where another program could listen. The
  port is held exclusively, so no other program can bind it as well, and is
  one the system picks unless `port` is set. A port that cannot be had is not
  swapped for another in silence.
- Each daemon run writes `bar-api.json` to the instance's data folder, which
  only the user can read, with the URL and a random 256-bit token, and deletes
  it on exit. A client sends the token in `hello`, within 2 s; the daemon
  compares it in constant time and sends nothing before. The token is never in
  a URL or a cookie, and handshake headers are never logged.
- A browser always sends an `Origin` with the handshake; a native client sends
  none, or any it likes, so the token stays the real gate. An `Origin` not in
  `[bar.websocket] origins`, compared as exact strings, is refused with 403.
  The list is empty by default, and `null` is never accepted, since any page
  can send it from a sandboxed frame.
- A `Host` other than `127.0.0.1:<port>` is refused, which stops DNS
  rebinding.
- It carries the same sessions, topics, limits and commands as the pipe.

The prompts browsers have begun to show before a public page reaches the
machine help, but are not relied on: a user can allow them, and a page served
from the machine itself is not asked about.

## Space for the Bar

A reservation says how far the bar reaches from an edge of the monitor. On
each edge, the tiling area keeps clear of whichever reaches further: the
system's work area or the bar.

- The system's work area already leaves out the taskbar and every bar that
  registers with the system as an app bar, and mosaico already follows it
  (`WorkAreaChanged`). Such a bar needs nothing more.
- A bar that registers and also reserves is never counted twice, since both
  reach the same line. A bar drawn below a taskbar on the same edge reserves
  down to its own bottom, and the taskbar is not counted again. So a bar can
  reserve without knowing what the system keeps.
- What reserves depends on the provider. Under `mosaico`, the built-in bar
  reaches its height below the top of the system's work area, as today, on
  each monitor it shows on. Under `yasb`, `zebar` and `external`, it is
  `[bar.reserve]` and what clients reserve through the API; one bar is active,
  so two reservations on an edge are the same strip, and the further counts.
  Under `none`, nothing.
- Through the API, a reservation names a monitor, or `all`, and how far the
  bar reaches from each edge it is on:
  `{"id":2,"reserve":{"monitor":"m-41c6e2","top":48}}`. Another one for the
  same monitor replaces it, and `release`, or all zeros, ends it.
- A reservation lasts as long as its session. When a session ends without
  releasing, its space is kept 5 s more, so a bar that restarts does not make
  the windows jump twice. `release`, a provider switch and `mosaico stop` end
  it at once.
- Reservations are refused with `not-provider` under `mosaico` and `none`, and
  with `too-large` past a quarter of the monitor on an edge.
- A reservation keeps tiles out of the strip, as the built-in bar's does. A
  window the user maximizes still covers it, as it covers the built-in bar;
  only a bar that registers with the system keeps maximized windows out too.
- A reservation follows its monitor by id. One for a monitor that disconnects
  is kept until its session ends, and applies again if the monitor returns.
- The tiling areas are worked out again, always from the system's work area
  and the reservations, on every work area change, display change, reload,
  reservation and session end, so nothing adds up. A reservation reads the
  system's work areas again first, as `retile` does, since an app that sets the
  work area without saying so is not reported. Each change retiles only the
  monitors whose tiling area moved.
- `mosaico doctor` shows per monitor and edge what the system keeps and what
  mosaico keeps, and warns when a named bar is connected and reserves nothing,
  which its integration always does.

## The Built-in Bar on the Same Model

The built-in bar stays inside the daemon, drawn from the state the API sends.

It stays inside because phase 35 depends on it: a click is hit tested against
what the bar last drew, stale bars are told apart by their `BarId`
generation, the click's action runs on the daemon's loop against the state the
bar shows, and whether a bar lets clicks through is decided from the desktop
on each update. In a process of its own, every click and redraw would cross a
process boundary and could act on a state the bar has not shown, and the
daemon would have a second program to start, watch and stop. What that would
bring is crash isolation, which the API makes possible later.

It is a client of the same model by construction:

- `BarState` is built by a pure function from `api::State` and a monitor's
  index, and the state builder takes over what `Tiler::bar_states()` and
  `Tiler::mark_maximized()` work out today, so all the built-in bar shows of
  mosaico is in the API.
- It takes the latest published state, as a session does, and redraws once
  per publish instead of after every action.
- Its clicks run their actions through the same path as an API `run`, from the
  same origin, so another bar can run whatever it runs.
- A test draws it from a state decoded from the API's JSON and from the state
  the tiler built, and expects the same content.

Machine readings (CPU, memory, media and time) are not in the state: they are
not mosaico's, and every bar reads them its own way.

## Architecture

```
Today:
Tiler::bar_states() -> Vec<BarState> -> BarManager::update -> BarRenderer    after every action, event batch and tick

With this phase, once per wake that changed something, while anything reads the state:
api::State::build(tiler, daemon facts, previous) -> Hub::publish(state, rev)  shares what did not change
    built-in bar (mosaico): BarState::from_state -> BarManager::update
    each session's writer thread: diff(last sent, latest) -> one update line  never on the daemon thread
session reader threads -> Task::Api(session, request) -> daemon loop             run, reserve, release
    run: Origin::Bar(index) -> carry_out -> reply after the publish that shows it
Reservations::resolve(provider, config, sessions, displays) -> Input::Reserve    in place of Input::Bar
```

With no session and no built-in bar, nothing builds the state at all.

### Core

- `config/bar_section.rs`: the `[bar]` table and the provider's check.
- `bar/provider.rs`: `BarProvider`, and what each one turns on: the built-in
  bar, which reservations count, launching.
- `bar/refresh.rs`: when the built-in bar refreshes for widgets that change
  with time, as a due time for the `Scheduler`. The tick thread goes, so
  nothing wakes the daemon each second unless such a widget shows.
- `tiler/reserve.rs`: `Reservations`, the reach per monitor and edge from each
  source and how they combine. `Input::Bar(BarReservation)` becomes
  `Input::Reserve`, and `Input::Displays` and `Input::WorkAreas` carry the
  same, so a tiling area is the system's work area kept clear of four reaches.
- `api/`, the bar API: `State`, shaped as the wire is; building it from the
  tiler's model and the daemon's facts, with unchanged parts shared; `diff`
  and `apply`, so `apply(diff(a, b), a) == b` can be tested for any two
  states; the protocol's messages and what is done with what is not known;
  one session's decisions as a pure state machine (what to send next, with
  which `seq`, `rev` and `base`, and when to drop it); the hub of the latest
  state and the sessions; the accept loop and session threads over the
  transport contracts, as `ipc::serve` is for the CLI; field paths; icons
  (36.6); and the WebSocket's handshake, frames and checks (36.9).
- Titles, programs and process ids are kept from when a window is taken in,
  and a title is read again on `TitleChanged` only while a session subscribes
  to `focus` or `windows`.
- `bar/programs.rs` (36.7): what to look for and run for each named bar, and
  the supervisor's restart rules, on an injected clock.

### Platform contracts

Core states each in behaviour terms, with a `# Contract for platform
implementations`. The platform's API names stay in the platform's docs.

- `api::ApiListener` and `api::ApiConnection`, for the daemon:
  - many sessions open at once, each for as long as its client wants, while
    the endpoint keeps accepting;
  - only processes of the same user on the same machine connect, elevated or
    not: no other users, no other machines, no sandboxed or low-integrity
    processes of the user's;
  - the daemon takes the endpoint's name first, or fails to open it, so no
    other program listens under it;
  - after a client's first message, the platform says whether it is the
    user's, from the client's own credentials, not from a process id, which
    can be spoofed;
  - a connection splits into a reader and a writer that work at once on two
    threads, so a read waiting for the client never holds up a write;
  - writes take a deadline, past which they fail and the connection closes;
  - the peer's process id, when the platform can tell, for doctor and the log
    only.

  Windows: overlapped pipe handles, since I/O on a synchronous handle is
  serialized and a pending read would block every write. An explicit security
  descriptor in place of the default, which gives Everyone and Anonymous read
  access: `O:<user>G:<user>D:P(D;;GA;;;NU)(A;;GA;;;SY)(A;;GA;;;<user>)S:(ML;;NWNR;;;ME)`.
  The owner is the user whether the daemon is elevated or not, network logons
  are denied, and the medium label keeps low-integrity processes from reading
  or writing. `PIPE_REJECT_REMOTE_CLIENTS` on every instance and
  `FILE_FLAG_FIRST_PIPE_INSTANCE` on the first, so a pipe someone else created
  first is never joined. After `hello`, `ImpersonateNamedPipeClient` and the
  client's token: the user's SID, at least medium integrity, no app container.
  The CLI pipe gets the same descriptor and flags, which `IpcServer`'s
  contract ("only clients of the same user") asks for already.
- `api::ApiClient`, for the CLI: connect, wait with `--wait`, and refuse an
  endpoint that is not the user's. Windows: the pipe opened with
  `SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION`, so a server that is not
  the daemon cannot act as the user, and its owner checked against the user's
  SID, as PuTTY and .NET's `CurrentUserOnly` do.
- `system::ParentWatch`, for `mosaico subscribe`: tell when the process that
  started this one exits.
- `tiler::Display` gains `bounds`, `name`, `device` and `scale`, each `None`
  when the platform cannot tell. Windows: `rcMonitor`, the target's friendly
  name, the source's GDI device name, and the monitor's DPI.
- `api::AppIcons` (36.6): a program's icon, as pixels at about a size,
  callable from any thread. Windows: the shell's icon for the executable, as
  the built-in bar draws it.
- `bar::BarPrograms` (36.7): find a named bar's program, start it detached
  from the daemon, tell when it exits, say whether it runs, and stop it: ask
  it to exit, wait, then end it.
- `api::LoopbackListener` (36.9): a TCP endpoint on the loopback address only,
  its port held exclusively. Windows: `SO_EXCLUSIVEADDRUSE`.
- Reused as they are: `Desktop::facts` for titles and programs,
  `WorkAreaChanged` for app bars, and `ProcessControl::is_alive` for doctor.

## Integrations

Each lives under `integrations/`, versioned with the API, and is described in
`docs/bar-api.md` and on the guide's new "Other Bars" page.

- **Zebar**, shipped. The widget pack is plain HTML and JavaScript with no
  build step, embedded in the binary, so setup writes it without a download.
  Its reducer applies snapshots and updates and checks `seq`. It draws the
  workspaces, layout and focused window of its own monitor in the colors of the
  `theme` topic. It is also published to Zebar's marketplace.
- **YASB**, upstreamed. The widgets follow YASB's contribution rules (a
  `BaseWidget` subclass, a Pydantic config, a docs page, ruff, pyright and
  pytest) and go to YASB as a pull request; the API's example files are its
  test fixtures.
- **Any bar that runs a command**: recipes for `mosaico subscribe --field` (a
  bar that reads lines), `--full` (one that reads JSON) and `mosaico query
  --field` on a timer (one that can only poll), with `mosaico action
  --monitor` for clicks and `[bar.reserve]` or `--reserve` for space.
- **Rainmeter**, a recipe: Rainmeter only reads a program's output once it
  ends, so a short PowerShell script runs `mosaico subscribe --field` and
  passes each value to the skin with `Rainmeter.exe !SetVariable`; space from
  `[bar.reserve]`.
- **Scripts**: a PowerShell example that reads `mosaico subscribe --full`.

## Migration and Compatibility

- A user who does nothing gets `provider = "mosaico"`: the same bar, on the
  same monitors, with the same space. An existing `config.toml` gains a
  comment block about `[bar]` (`merge_missing_config_sections`), with no value
  set.
- `enabled = false` in `bar.toml` still means no bar and no space.
- `mosaico debug state` stays a dump for people debugging mosaico, which may
  change in any release; `mosaico query` is the API. `bar_reservation.rs` keeps
  reading `debug state`.
- The CLI protocol stays at 2. `Status` gains optional fields (the provider,
  the sessions and their clients, the space per monitor), which an older CLI
  ignores, and which a newer one finds missing from an older daemon.
- The bar API starts at 1, and `mosaico query` and `subscribe` work across
  releases that speak it.
- The CLI pipe's new descriptor keeps out only other users, which could read
  it but never send it a request.

## Edge Cases

1. A bar that starts before the daemon waits with `--wait`, as the
   integrations do; the daemon never connects out to a bar, so there is no
   subscription to lose. When the daemon restarts, `bye` comes, the session
   ends, and the next one shows a new `boot` and a fresh snapshot.
2. A bar that crashes: its session ends, and its space 5 s later unless it
   reserves again first. With `launch`, it is started again.
3. A provider switch during a drag or a transition (phases 34 and 35): the
   retile takes the usual path, and a window the user holds is left alone until
   it is dropped.
4. A display change: monitors keep their ids when their key is known, so
   reservations and `--monitor` still mean the same display. One that cannot
   be identified comes back with a new id, and the reservations on the old one
   end with their sessions. A command for a monitor that just went is answered
   `no-monitor`.
5. Global workspace mode: a workspace command switches every monitor and gives
   focus to the one it names, as a click on the built-in bar does.
6. Hotkeys paused: commands still run, and `toggle-pause` from a bar resumes
   them.
7. A client that sends nothing, or a line over 1 MiB, is dropped; one that
   sends a malformed request gets `malformed`, and the session goes on.
8. An elevated daemon: the pipe's owner is the user, so a bar that is not
   elevated connects. Since the user's processes then send requests into an
   elevated one, no request runs a program or names a file.
9. A pipe someone else created first under the endpoint's name: the API does
   not open, the log and doctor say why, and clients refuse that pipe, since
   its owner is not the user.
10. `mosaico subscribe` left behind by a bar that stopped reading exits on its
    next failed write, or when the program that started it exits.
11. `config.toml` and `bar.toml` saved together with a switch to `mosaico`: the
    switch reads `bar.toml` fresh, so the order of the reloads does not matter.
12. A bar killed while it was registered with the system: Windows can keep its
    space out of the work area until Explorer works the areas out again or
    restarts. mosaico follows the work area either way, and the guide says how
    to get the space back.
13. Explorer restarts: the work areas change, and mosaico follows them as it
    does today. A reservation through the API does not depend on Explorer, so
    it holds.
14. The WebSocket's port is taken: the WebSocket is not opened, the log says
    why, and the pipe still works.

## Testing

In core, against fakes, with no pipe, process or desktop:

- The provider check and what each provider turns on: under any provider but
  `mosaico`, a fake renderer and machine see no bar made, nothing read and no
  refresh due, and a broken `bar.toml` is neither reported nor changed. A
  switch on a reload, in every direction, retiles once.
- `Reservations`: every source and edge, an app bar and a reservation on the
  same edge counted once, a bar below a taskbar, the quarter limit, a monitor
  gone and back, the grace, the refusals.
- The state builder against the tiler `Harness`, and `diff` and apply as a
  property: for random pairs of states and every set of topics, applying the
  difference gives the second state, sharing what did not change.
- Sessions over in-memory connections: versions, unknown topics and requests,
  `seq`, `rev` and `base`, `resync`, a reply after its update, scope refusals,
  `bye`, the limits, a client that never says `hello`, and a writer that
  blocks: the daemon keeps publishing, the session is dropped at its
  deadline, and one that was only slow gets a single update to the latest
  state.
- The example files, decoded and encoded back byte for byte;
  `BarState::from_state` against `Tiler::bar_states()` on the same desktops
  before the latter goes; the built-in bar drawn from decoded JSON.
- CLI commands with the fake `World`: `query`, `subscribe` with each option,
  `--wait` across a restart, exit codes, `action --monitor`. The `launch`
  supervisor on a fake clock: a bar already running, restarts and their
  delays, giving up, stopping only what it started.
- The WebSocket's checks, from recorded handshakes of a browser, a WebView and
  a native client, with the origins and hosts that must fail (`null`,
  `http://tauri.localhost.example.com`, other cases and ports).

In `mosaico-windows`, with pipes, processes and windows of the tests' own: the
API pipe on a unique name (many sessions, a read and a write at once, write
deadlines); both pipes' security (the descriptor read back with
`GetSecurityInfo`, a pipe the test creates first keeping the server from
opening and refused by the client, a thread on a low-integrity copy of the
test's token turned away); `Display`'s new fields; `AppIcons` for the test's
own executable; `BarPrograms` with a small program the tests start and stop.

On demand, on the desktop, with the user's daemon stopped:
`tests/external_bar.rs` runs a test daemon with `provider = "external"`, has a
client of its own reserve 48 px at the top of the primary monitor and checks
the tiling area, ends the session and checks the space comes back after the
grace, then registers a window of its own as an app bar, which changes the
work area of the live desktop, and checks that mosaico follows it. It joins
the desktop suites in the testing skill.

Metrics:

- Every existing scenario keeps its counts with the built-in bar on and no
  client: the state is built, and nothing more is asked of the platform.
- New scenarios: from `mosaico` to `external` and back (a retile each, and no
  bar drawn while `external`); a workspace switch and a burst of title changes
  with no client, a `monitors` client and a `windows` client (titles read with
  the last only); and a reservation (one retile, of its monitor).
- `metrics/api.toml`, a new baseline: the messages and bytes each session is
  sent in those scenarios, so an update that grows, or a snapshot sent where an
  update would do, shows in review.

## Later

- A `focus-window` action that names its window, so a bar can list windows and
  focus one on a click, as a taskbar does.
- A `system` topic with the readings the built-in bar takes (CPU, memory,
  media), read only while a session subscribes.
- Names for workspaces in the config; the API has the field already.
- The built-in bar as a process of its own on the API, if crash isolation is
  ever worth its cost.
- A client library: the API's types as a crate of their own, and the Zebar
  pack's reducer as a module for other web bars.
- A mosaico provider inside Zebar, in its own code, and a Rainmeter plugin, if
  their users ask.
- A JSON Schema and TypeScript types generated from the example files.
- Running a command on an event, for scripts that want hooks rather than a
  session.
- The same contracts on a socket, for `mosaico-linux` and `mosaico-macos`.

## Open Questions

For the owner to decide. Each carries the plan's proposal.

1. Where the provider lives: `[bar] provider` in `config.toml` (proposed), or a
   key in `bar.toml`.
2. A provider mosaico does not know: read as `external` (proposed, so the
   built-in bar never draws over the user's), or as `mosaico`.
3. `launch`: off by default (proposed), or on for `yasb` and `zebar`.
4. Whether `mosaico bar setup` also sets the provider, so one command switches
   bars live (proposed), or only writes the bar's files.
5. YASB: write the `mosaico` widgets and offer them upstream (proposed), and
   whether setup prints the polling fallback until a YASB release has them
   (proposed), or nothing.
6. Zebar: embed the widget pack in the binary (proposed, its size measured in
   its PR), or fetch it at setup, as the community rules are; and publish it to
   Zebar's marketplace too (proposed).
7. The WebSocket (36.9): keep it planned and build it only when a bar needs it
   (proposed), build it in this phase, or drop it.
8. The endpoint's name: fixed, `mosaico.bar` (proposed), or with the user's SID
   in it, which no other user could block by taking it first, at the cost of a
   name every bar must look up.
9. Reservations through the API under `mosaico`, for a dock at another edge:
   refused (proposed, one bar at a time), or taken.
10. Monitor ids derived from `MonitorKey` (proposed), or the key itself on the
    wire.
11. The numbers: 5 s of grace after a session ends, a 10 s write deadline, 16
    sessions, 32 waiting requests, a quarter of a monitor per reservation,
    titles cut at 1,000 characters.
12. A Cargo feature to build without the built-in bar. The plan says no: one
    binary ships, so it would serve source builds only and double the CI
    matrix. 36.1 measures what the bar costs, so the answer rests on a number.

## New and Modified Files

New code goes in new files where the file it belongs next to is near or over
the 150-line limit after phase 35 (`bar/manager.rs`, `tiler/desktop.rs`,
`config/reload/mod.rs`, `config/store/merge_config.rs`, `daemon_loop.rs`,
`daemon_loop_commands.rs`, `daemon_threads.rs`, `monitor.rs`, `ipc.rs`,
`main.rs`).

```
crates/
  mosaico-core/src/
    config/bar_section.rs        # NEW: [bar] in config.toml, the provider check
    config/template_config.rs, config/store/, config/reload/  # template, hint, bar.toml only under mosaico
    bar/provider.rs, bar/refresh.rs  # NEW: what each provider turns on; the refresh due time
    bar/state.rs                 # BarState::from_state
    bar/programs.rs              # NEW (36.7): named bars' programs, the supervisor
    tiler/reserve.rs             # NEW: Reservations
    tiler/input.rs, tiler/work_area.rs, tiler/bar_states.rs, tiler/desktop.rs  # Input::Reserve, Display
    daemon/scheduler.rs          # the refresh and grace due times
    system/parent_watch.rs       # NEW (36.4)
    api/                         # NEW: the bar API
    ipc/response.rs, instance.rs # DaemonStatus fields, the API endpoint's name
  mosaico-windows/src/
    api_pipe.rs, pipe_security.rs  # NEW: the API pipe, and the descriptor both pipes use
    ipc.rs, monitor.rs, monitor_identity.rs  # the CLI pipe's security; Display's new fields
    parent_watch.rs, icons.rs, bar_programs.rs, loopback.rs  # NEW (36.4, 36.6, 36.7, 36.9)
    daemon_loop*.rs, daemon_threads.rs, daemon_types.rs  # the provider, the hub, Task::Api, no tick
  mosaico/src/
    main.rs, platform.rs, commands/doctor_bar.rs  # the new commands and doctor lines
    commands/query.rs, commands/subscribe.rs, commands/bar_setup.rs  # NEW
  mosaico/tests/external_bar.rs  # NEW: the on-demand desktop suite
  mosaico-metrics/, xtask/       # the api.toml baseline
integrations/zebar/, integrations/yasb/  # NEW (36.8)
metrics/tiler.toml, metrics/api.toml     # the new scenarios and baseline
docs/
  bar-api.md, bar-api/           # NEW: the API for bar authors, and its example files
  status-bar.md, configuration.md, ipc.md, daemon.md, cli.md, monitor-management.md
website/src/
  guide/other-bars.md            # NEW: providers, YASB, Zebar, any bar
  guide/status-bar.md, guide/configuration.md, guide/cli.md, SUMMARY.md
```

## Tasks

Every step runs fmt, clippy, the safe test suites and `cargo xtask metrics`,
records its metrics and scenarios with `cargo xtask metrics --bless`, and
brings its docs, templates and config hints, so it can merge on its own. The
phase starts once phase 35 is on `main`. 36.1 comes first, since the rest
hangs off the provider; 36.2 needs 36.1, and 36.3 needs 36.2; 36.4, 36.5 and
36.6 need 36.3; 36.7 needs 36.4 and 36.5; 36.8 needs 36.7, and 36.6 for
icons; 36.9 needs 36.3 and is optional.

### 36.1 The bar provider

- [ ] `[bar] provider` in `config.toml` with `mosaico`, `external` and `none`,
      read with a check as themes are, an unknown value reported and read as
      `external`; `yasb` and `zebar` come with 36.7
- [ ] The daemon holds the built-in bar only under `mosaico`: otherwise no bar
      manager, no `bar.toml` read, merged or watched, no machine readings
- [ ] The refresh for widgets that change with time as a due time in the
      `Scheduler`, in place of the tick thread
- [ ] `Display` gains `bounds`, in the contract and on Windows
- [ ] `Reservations` and `Input::Reserve` in place of `BarReservation`: the
      reach from four edges per monitor, kept clear of with the system's work
      area, the further counting
- [ ] `[bar.reserve]` and its per-monitor tables, under `external`
- [ ] Switching providers on a reload in one step, in every direction
- [ ] doctor: the provider, `bar.toml` not read and why, settings the provider
      ignores, and what the system and mosaico keep per monitor and edge
- [ ] The template, the hint for existing files, `configuration.md`,
      `status-bar.md` and the guide
- [ ] Tests as listed under Testing; metrics scenarios for the switches
- [ ] What the built-in bar adds to the binary, measured with `cargo bloat` and
      recorded in the PR

### 36.2 The state model

- [ ] `api::State` and its parts, built from the tiler and the daemon, with
      unchanged parts shared, and revisions
- [ ] `diff` and apply, with the property tests
- [ ] `Display` gains `name`, `device` and `scale`, in the contract and on
      Windows
- [ ] `BarState::from_state`, the built-in bar redrawn once per publish, and
      what `Tiler::bar_states()` worked out moved into the builder
- [ ] The example files in `docs/bar-api/` and their test
- [ ] Every existing metrics scenario unchanged

### 36.3 Bar API sessions

- [ ] The listener and connection contracts, and the API pipe on Windows:
      overlapped I/O, the security descriptor, remote rejection, the first
      instance flag and the client check; the same security for the CLI pipe
- [ ] The hub, sessions, `hello` and `welcome`, topics, capabilities,
      snapshots and updates with `seq`, `rev` and `base`, `subscribe`,
      `resync`, `ping` and `bye`
- [ ] Back-pressure: writer threads, the write deadline, the session and
      request limits
- [ ] `run`, from `Origin::Bar`, with the scope check, and its reply after its
      update
- [ ] `docs/bar-api.md`, and the API in `ipc.md` and `daemon.md`
- [ ] Tests as listed; metrics scenarios with a client, and `metrics/api.toml`

### 36.4 `mosaico query` and `mosaico subscribe`

- [ ] The client contract and its Windows pipe client, with the owner check
- [ ] `query` with `--field`; `subscribe` with `--topics`, `--monitor`,
      `--full`, `--field`, `--wait` and `--requests`, and its exits
- [ ] `ParentWatch` and its Windows implementation
- [ ] `mosaico action --monitor`
- [ ] doctor: the sessions and their clients
- [ ] `cli.md` and the guide's CLI page; tests with the fake `World`

### 36.5 Space through the API

- [ ] `reserve` and `release`, the space ending with its session after the
      grace, the refusals, monitors followed by id, and the system's work areas
      read again first
- [ ] `mosaico subscribe --reserve`
- [ ] doctor: what each session reserves
- [ ] Tests; a metrics scenario for a reservation
- [ ] The desktop suite `tests/external_bar.rs`, built here and run on demand

### 36.6 Icons

- [ ] `AppIcons` and its Windows implementation, icon keys in `focus` and
      `windows`, and `icon` requests answered with a PNG, cached per key and
      size
- [ ] Tests, the PNG decoded back among them

### 36.7 YASB and Zebar

- [ ] `yasb` and `zebar` as providers, with their defaults
- [ ] `launch` and `program`: `BarPrograms`, the supervisor, and stopping a bar
      mosaico started at `mosaico stop`, at shutdown and on a switch away
- [ ] `mosaico bar setup yasb` and `mosaico bar setup zebar`, with
      `--no-switch`
- [ ] doctor, per bar: installed, running, set up and current, connected, and
      a warning when it is connected and reserves nothing
- [ ] Tests for the supervisor and setup; the Windows contract with a small
      program of the tests' own

### 36.8 Integrations

- [ ] The Zebar widget pack in `integrations/zebar/`, embedded and written by
      setup, and published to Zebar's marketplace
- [ ] The YASB widgets in `integrations/yasb/`, offered to YASB upstream, and
      the fallback setup prints until a YASB release has them
- [ ] Recipes for any bar that runs commands, for Rainmeter and for scripts
- [ ] The guide's "Other Bars" page, and its status bar page corrected (it
      shows a `monitor` key and a `[[center]]` side that do not exist)

### 36.9 Local WebSocket (optional)

- [ ] `[bar.websocket]`, off by default, with `port` and `origins`
- [ ] `LoopbackListener` and its Windows implementation
- [ ] The handshake and frames in core, the connection file, and the token,
      origin and host checks, on the same sessions
- [ ] Tests from recorded handshakes; the size it adds, measured in the PR

### Verification

- [ ] Manual test: switch providers live, `mosaico` to `zebar` to `none` to
      `mosaico`, and see windows move once each time and nothing left behind
- [ ] Manual test: YASB and Zebar on every monitor, in both workspace modes,
      following switches, layouts, monocle and focus, and clicks on each acting
      on its own monitor without moving the pointer
- [ ] Manual test: a bar started before the daemon, the daemon restarted under
      it, Explorer restarted, a bar killed with `launch` on and off, and
      `mosaico stop` with a bar mosaico started and one the user started
- [ ] Manual test: a client suspended mid-session never slows the hotkeys or
      the built-in bar, and gets one update once resumed
- [ ] Run `tests/external_bar.rs` on the desktop
- [ ] Update `.plans/plan.md`

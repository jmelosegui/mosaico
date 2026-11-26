# Phase 39: Lua Scripting and Events

**Status:** In progress

**Goal:** Let users react to what mosaico does and automate it, with Lua scripts
that a `mosaico script run` process runs as a client of the daemon, and an event
stream any program can subscribe to.

## Overview

Today a client can only ask the daemon one thing at a time: run an action, show
its status, dump its state. Nothing tells a client that focus moved, a workspace
changed or a monitor went away, and every command acts on the focused window.
Users who want "float the meeting window on the second monitor" or "switch
layouts when I dock" have no way to write it.

This phase adds three layers, each usable on its own:

1. **Events.** A `subscribe` IPC command keeps its connection open and streams
   desktop events, decided in core by comparing the desktop before and after
   each daemon step. `mosaico events` prints them, so any language can use them.
2. **Targeted commands.** Commands that name the window, monitor or workspace
   they act on, so a script is not limited to the focused window.
3. **Scripts.** `mosaico script run <name>` loads a Lua file and gives it a
   `mosaico` table: read the state, run actions and commands, handle events,
   set timers, start programs. The script runs in its own process, never in the
   daemon.

## Design Goals

- **Nothing in the daemon.** The daemon gains a subscription and a few commands;
  it never loads Lua. A script that errors, loops or crashes only stops its own
  process.
- **One binary.** Scripts run through `mosaico.exe`, like every other client.
- **Portable.** Events, commands and the scripting API are defined in core and
  in an OS-agnostic `mosaico-script` crate; the platform only carries bytes and
  starts processes.
- **Testable.** The script runtime takes its IPC client, clock, launcher and
  process runner as parameters, so scripts are tested against fakes.
- **Old clients keep working where they can.** The protocol version is bumped
  once, for all new commands together.

## Events (`mosaico-core/src/desktop_events/`)

```rust
/// Something that changed on the desktop, as a subscriber sees it.
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DesktopEvent {
    WindowManaged { window: WindowId, monitor: MonitorId, workspace: usize, class: String, title: String },
    WindowUnmanaged { window: WindowId },
    WindowMoved { window: WindowId, monitor: MonitorId, workspace: usize },
    WindowTitleChanged { window: WindowId, title: String },
    WindowFloatingChanged { window: WindowId, floating: bool },
    FocusChanged { window: Option<WindowId>, monitor: MonitorId },
    WorkspaceChanged { monitor: MonitorId, workspace: usize },
    LayoutChanged { monitor: MonitorId, workspace: usize, layout: LayoutKind, monocle: bool },
    MonitorAdded { monitor: MonitorId },
    MonitorRemoved { monitor: MonitorId },
    Paused { paused: bool },
    ConfigReloaded,
}
```

- `fn changes(before: &DesktopSnapshot, after: &DesktopSnapshot) -> Vec<DesktopEvent>`
  is a pure function in core. Every event is derived in one place, so the tiler
  is not sprinkled with emit calls and a new event is one diff and one test.
- `WorkspaceSnapshot` gains the workspace's layout name and monocle flag, which
  the diff needs.
- The daemon builds a snapshot after a step only while someone is subscribed.
- Events report what happened. A subscriber cannot veto or change a decision.
- `EventKind` names each variant (`"focus_changed"`), parses from those names and
  rejects unknown ones, so a typo in a script fails at once instead of
  registering a handler that never fires.

## Subscription over IPC

- `Command::Subscribe { events: Vec<EventKind> }` (empty means all). The daemon
  replies `ok` with the current `DesktopSnapshot`, then writes one
  `DesktopEvent` per line on the same connection until either side closes it.
- The transport contract changes from "one command and one reply per
  connection" to "one command and one reply, except a subscription, which then
  carries events until either side closes it".
- `serve` hands a subscribed connection to a `Subscribers` registry and goes on
  to the next client, so a subscriber never holds up other commands.
- Each subscriber has a bounded queue (256 events) and its own writer thread. A
  subscriber that falls behind is sent `{"event":"lagged"}` and dropped, so a
  stuck script never slows the tiler.
- The daemon closes every subscription when it stops.
- `Command::State` returns the snapshot without the debug text of `Inspect`.
- `PROTOCOL_VERSION` goes from 2 to 3.
- `mosaico events [--json] [<kind>...]` prints the stream. `mosaico debug events`
  stays as it is: it reads the OS directly and needs no daemon.

## Targeted commands

`Command::Order { order }` carries a `tiler::Order` that names what it acts
on instead of the focused window or monitor. Workspaces are counted from 0,
as in the snapshot and the events.

| Order | Does |
| --- | --- |
| `focus { window }` | Focuses the window and brings it into view (`Tiler::raise`, from phase 38). |
| `send { window, workspace }` | Focuses a tiled window, sends it to a workspace of its monitor and follows it, as `send-to-workspace` does. |
| `float { window, floating }` | Focuses the window, then floats or tiles it if it is not already. |
| `close { window }` | Asks the window to close, without focusing it. |
| `show { monitor, workspace }` | Shows a workspace on a monitor; focus stays where it is. In global mode every monitor switches. |
| `layout { monitor, workspace, layout }` | Sets a workspace's layout, on screen or not. |
| `monocle { monitor, on }` | Turns monocle on or off on the workspace a monitor shows, focusing that monitor. |

An order naming a window, monitor or workspace the daemon does not have is
answered with `ErrorCode::NotFound` and not carried out.

## Script runtime (`crates/mosaico-script`)

A new OS-agnostic crate the CLI depends on. It embeds Lua 5.4 through `mlua`
(feature `vendored`, so no Lua install is needed).

```lua
mosaico.state()                         -- monitors, workspaces, windows, focus
mosaico.windows{ class = "Code", title = "part", workspace = 2, monitor = id, floating = true }
mosaico.action("goto-workspace-3")      -- any action name the CLI accepts
mosaico.focus(window) mosaico.send(window, workspace) mosaico.float(window [, on])
mosaico.close(window) mosaico.show(monitor, workspace)
mosaico.layout(monitor, workspace, "three-column") mosaico.monocle(monitor [, on])
mosaico.on("focus_changed", function(event) ... end)
mosaico.wait("window_managed" [, { class = "Code" }] [, seconds])
mosaico.after(seconds, fn) mosaico.every(seconds, fn) mosaico.cancel(timer)
mosaico.sleep(seconds) mosaico.stop([code])
mosaico.launch{ "wt.exe", "-d", "C:/src" }
mosaico.exec{ "git", "status", timeout = 5 }  -- { ok, code, stdout, stderr, timed_out }
mosaico.log([level,] text) mosaico.json.encode(v) mosaico.json.decode(s)
mosaico.args                            -- key=value arguments from the command line
```

Workspaces are counted from 1 in scripts, as the bar and the
`goto-workspace-N` actions count them; the runtime converts to the daemon's
0-based index and back. Window and monitor ids are the integers the daemon
reports.

- Calls that ask the daemon return `value` or `nil, err`; they never raise, so
  a script decides what a failure means.
- Handlers and timers run one at a time on one thread. A handler that waits
  holds up the others, as in any single-threaded event loop.
- A script with no handlers or timers exits when its file has run. One with
  handlers keeps running until `mosaico.stop`, Ctrl+C, or the daemon stops.
- An error in a handler is logged and the script goes on; `--fail-fast` exits.
- `io.popen` and `os.execute` are removed: `mosaico.exec` takes an argument list
  and runs no shell. `require` searches only `lib\` in the config folder.
- The runtime takes an `IpcClient`, a clock, a `Launcher` and a `ProcessRunner`,
  so every function is tested against fakes.

## `mosaico script` commands

| Command | Does |
| --- | --- |
| `mosaico script run <name or path> [key=value...]` | Runs a script. A bare name is looked up in `scripts\`. |
| `mosaico script new <name>` | Writes `scripts\<name>.lua` from a commented template and opens it. |
| `mosaico script edit <name>` | Opens a script. |
| `mosaico script list` | Lists the scripts in `scripts\`. |
| `mosaico script add <path>` | Copies a script into `scripts\`. |
| `mosaico script open` | Opens the scripts folder in the file manager. |
| `mosaico script autostart enable|disable <name>` | Runs a script at logon, in the background. |

Files live in the config folder (`%USERPROFILE%\.config\mosaico`, or
`MOSAICO_CONFIG_DIR`):

```
scripts\        scripts for `mosaico script run`
lib\            modules for `require`, and mosaico.lua with the API's
                type annotations for editors that read LuaLS annotations
.luarc.json     points the Lua language server at lib\
```

`new` writes `lib\mosaico.lua` and `.luarc.json` when they are missing, so an
editor offers completion for the API from the first script.

### Autostart

A script registered with `script autostart enable` gets its own logon task,
`Mosaico script <name> for <user>`, that runs `mosaico script run <name>
--background`. That starts the script again detached, without a console, writing
to `logsscripts<name>.log` in the config folder, and returns. A new
`ScriptAutostart` contract sits beside `Autostart`, which keeps its single
registration for mosaico itself. A script started before the daemon
waits for it; when the daemon stops, `script run` waits for it to come back and
subscribes again, so a restart does not stop autostarted scripts.

## Decisions

1. **Lua runs only in `mosaico script run` processes,** never in the daemon.
   Keybindings that run a script use a `[[launch]]` binding with
   `command = "mosaico.exe"` and `args = ["script", "run", "<name>"]`.
2. **Lua 5.4 via `mlua` with `vendored`.** The most used embedding in Rust, no
   runtime to install. Lua 5.4 rather than LuaJIT, which stays on the 5.1 language.
3. **The 2,000,000-byte ceiling is removed.** `cargo xtask size` still prints the
   size so growth stays visible in CI logs.
4. **Events are derived by comparing snapshots,** not emitted from tiler code
   paths. One place to test, and the tiler stays unaware of subscribers.
5. **Events are notifications only.** No hook can change what the daemon does.
6. **`mosaico script run`, not `mosaico do` or `mosaico run`,** so it reads as a
   script and cannot be confused with `mosaico start`. `mosaico script open`
   opens the folder.
7. **No Lua config file.** `config.toml` and the other TOML files stay the only
   configuration.
8. **The settings app does not manage scripts in this phase.** The tray gains an
   "Open scripts folder" item.

## Tasks

Each task is one pull request, stacked on the one before, and closes its issue.

### 39.1 Plan

- [x] This file and the roadmap row

### 39.2 Remove the binary size ceiling (#243)

- [x] `cargo xtask size` reports the size and no longer fails over a ceiling
- [x] `AGENTS.md` and the testing skill updated

### 39.3 Desktop events in core (#244)

- [x] `WorkspaceSnapshot` carries layout and monocle
- [x] `DesktopEvent`, `EventKind` and `changes(before, after)`, with a test per event

### 39.4 Event subscription (#245)

- [x] Transport contract and `serve` hand subscriptions to `Subscribers`
- [x] Bounded queues, lagging subscribers dropped, all closed at stop
- [x] `Command::Subscribe` and `Command::State`; protocol version 3
- [x] The Windows daemon publishes changes after each step while subscribed
- [x] `mosaico events`

### 39.5 Targeted commands (#246)

- [x] The orders in the table above, with tiler tests on the fake platform

### 39.6 Script runtime (#247)

- [x] `mosaico-script` crate, `mlua` with Lua 5.4
- [x] The `mosaico` table, event loop, timers, sandbox, `require` from `lib\`
- [x] Tests against a fake daemon, clock, launcher and process runner

### 39.7 `mosaico script` commands (#248)

- [x] `run`, `new`, `edit`, `list`, `add`, `open`
- [x] Template, `lib\mosaico.lua` annotations, `.luarc.json`
- [x] Waiting for and resubscribing to the daemon

### 39.8 Script autostart and tray (#249)

- [x] A `ScriptAutostart` contract beside `Autostart`; `script autostart`
- [x] Tray "Open scripts folder"

### 39.9 Documentation and verification (#250)

- [x] `website/src/guide/scripting.md` with examples, `cli.md` updated
- [x] Manual check against a separate instance: `mosaico events` while
      switching workspaces and opening windows; a handler script; each order
      from a script; `script run --background` and `script autostart
      enable` surviving a daemon restart and a sign-in; the tray item

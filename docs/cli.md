# CLI Command System

Mosaico exposes a `clap`-based command-line interface as the sole user entry
point. All interactions with the window manager -- starting the daemon, sending
actions, debugging -- go through this CLI.

## Architecture

The CLI is defined in the `mosaico` crate (`crates/mosaico/`). It uses
`clap::Parser` for argument parsing and dispatches each subcommand to a
dedicated handler module under `crates/mosaico/src/commands/`.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico/src/main.rs` | Top-level CLI definition and dispatch |
| `crates/mosaico/src/action_commands.rs` | `ActionCommands`, `DirectionCommands`: the `action` subcommands and the action each names |
| `crates/mosaico/src/commands/mod.rs` | Module declarations for all commands |
| `crates/mosaico/src/commands/init.rs` | `mosaico init` handler |
| `crates/mosaico/src/commands/start.rs` | `mosaico start` handler |
| `crates/mosaico/src/commands/stop.rs` | `mosaico stop` handler |
| `crates/mosaico/src/commands/status.rs` | `mosaico status` handler |
| `crates/mosaico/src/commands/doctor.rs` | `mosaico doctor` handler |
| `crates/mosaico/src/commands/doctor_animations.rs` | What `mosaico doctor` says about window transitions |
| `crates/mosaico/src/commands/doctor_bar.rs` | What `mosaico doctor` says about `bar.toml`: whether it loads, and the clicks on the bar's widgets |
| `crates/mosaico/src/commands/autostart.rs` | `mosaico autostart enable` / `disable` / `status` handler |
| `crates/mosaico/src/commands/daemon.rs` | `mosaico daemon` handler (hidden) |
| `crates/mosaico/src/commands/action.rs` | `mosaico action <verb>` handler |
| `crates/mosaico/src/commands/pause.rs` | `mosaico pause` / `mosaico unpause` handler |
| `crates/mosaico/src/commands/banner.rs` | Shared ASCII logo used by `start` and `doctor`, and the `mosaico banner` handler |
| `crates/mosaico/src/commands/update.rs` | `mosaico update` handler |
| `crates/mosaico/src/commands/debug/list.rs` | `mosaico debug list` handler |
| `crates/mosaico/src/commands/debug/events.rs` | `mosaico debug events` handler |
| `crates/mosaico/src/commands/debug/move_window.rs` | `mosaico debug move` handler |
| `crates/mosaico/src/commands/debug/state.rs` | `mosaico debug state` handler |

### Command Context

Commands do not reach for the OS themselves. They take a
`commands::context::Context` holding everything they use: the daemon (through
`mosaico_core::ipc::IpcClient`), process control
(`mosaico_core::system::ProcessControl`), start at login
(`mosaico_core::system::Autostart`), window control
(`mosaico_core::WindowControl`), the windows and monitors on the desktop
(`mosaico_core::tiler::Enumerator`), the desktop as the tiler reads it
(`tiler_desktop`, a `mosaico_core::tiler::Desktop` made the first time a
command asks for it, which also says whether the system lets windows be
animated), the path of the running executable
(`current_exe`, which is what autostart registers and what `update`
replaces), file checks that never wait on the network
(`mosaico_core::system::FileProbe`, for the autostart check), the
instance's `PidFile`, a clock, the HTTP client that looks up and downloads
releases (`mosaico_core::http::HttpClient`), how this platform's release is
packaged and installed (`mosaico_core::update::Installer`), the logo, and
the output and error streams. They return an `Exit` (`Success` or
`Failure`), and only `main` calls `std::process::exit`.

`crates/mosaico/src/platform.rs` builds the real context, and the `Startup`
that the `daemon` subcommand hands to `mosaico_core::lifecycle::run()`; it is
the one place in the CLI that picks a platform implementation. Before any
command runs, `main` sets the process up through `platform::init()`, which
makes it DPI aware, so the coordinates commands move windows to or show match
the daemon's. The real context makes the tiler's `Desktop`, from a
`Win32Platform`, only when a command asks for it, as only `doctor` does. The
unit tests build one with the `Harness` in `commands/test_support/harness.rs`,
from `commands/test_support.rs`, a fake
`World` standing in for the daemon, processes, registry, program files,
desktop, GitHub, the release installer and clock, and
`commands/fake_desktop.rs`, a fake list of windows and monitors, so `start`,
`stop`, `status`, `autostart`, `action`, `pause`, `update`, `debug move`,
`debug list` and `doctor`'s autostart, daemon, monitor and window transition
checks are tested without a daemon, a desktop or a network. `daemon` is tested
with a fake lock and loop.

No command calls the platform crate itself. The two that run without a
context get what they need from `platform.rs` as well: `daemon` its
`Startup` from `platform::daemon_startup()`, and `debug events` its event
source, Ctrl+C and window titles from `platform::event_source()`,
`platform::interrupts()` and `platform::desktop()`.

### Key Types

- `Cli` -- top-level `clap::Parser` struct
- `Commands` -- enum of all subcommands: `Init`, `Start`, `Stop`, `Status`,
  `Doctor`, `Autostart`, `Update`, `Action`, `Banner`, `Pause`, `Unpause`,
  `Debug`, `Daemon`
- `ActionCommands` -- enum: `Focus { direction }`, `Move { direction }`,
  `Retile`, `ToggleMonocle`, `CloseFocused`, `CycleLayout`,
  `GoToWorkspace { n }`, `GoToWorkspaceNext`, `GoToWorkspacePrev`,
  `SendToWorkspace { n }`; `action()` gives the `Action` each names
- `DirectionCommands` -- enum: `Left`, `Right`, `Up`, `Down`
- `DebugCommands` -- enum for debug sub-subcommands: `List`, `Events`, `Move`,
  `State`
- `AutostartCommands` -- enum: `Enable`, `Disable`, `Status`

## Available Commands

### `mosaico init`

Creates default configuration files in `~/.config/mosaico/`:

- `config.toml` -- layout, border, and logging settings
- `keybindings.toml` -- keyboard shortcuts
- `rules.toml` -- window management rules
- `user-rules.toml` -- personal rule overrides, never overwritten
- `bar.toml` -- status bar settings

Skips files that already exist to avoid overwriting user customizations.

### `mosaico start`

Launches the daemon as a detached background process. Re-executes the current
binary with the hidden `daemon` subcommand using Windows process creation flags
`CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW` (constant `DETACH_FLAGS =
0x08000200`) to fully detach from the terminal.

On success, prints a startup banner showing the ASCII logo, config directory,
PID, repository URL, and a rotating tip. Tips cycle based on the current
timestamp so users see a different one each launch. After the banner, a
version check queries the GitHub releases API (`update::newer_release` in
`mosaico-core`, through the platform's HTTP client) and prints an update
notice if a newer release exists.

If a daemon is already running (detected via IPC pipe check), it reports the
existing state instead of starting a second instance.

`mosaico start --quiet` prints nothing and skips the version check, so it
returns as soon as the daemon answers. Autostart runs it this way, so login
never waits on the network. Errors are still printed to stderr.

### `mosaico stop`

Sends a `Stop` command to the running daemon over IPC (named pipe). If the
IPC pipe is unavailable (e.g. the IPC thread crashed), falls back to reading
the PID file and terminating the process via `ProcessControl::kill()`. Reports
success or failure.

### `mosaico status`

Asks the daemon for its status and reports its version and how many windows
it manages. It suggests restarting with `mosaico stop` and `mosaico start`
when the daemon is another release, and when it answers without a status
(only daemons from before the protocol was versioned do). A daemon that is
listening but does not reply is reported as not responding.

Other commands suggest the same restart when the daemon closes the
connection without replying, which is what a daemon from before the
protocol was versioned does with a request it cannot read.

When no daemon answers, it reads the PID file: a process that is still alive
is reported as not responding, and a PID file left by a dead process is
removed.

### `mosaico doctor`

Runs a comprehensive health check, printing colored status tags for each
check:

| Tag | Color | Meaning |
|-----|-------|---------|
| `[ok]` | Green | Check passed |
| `[warn]` | Yellow | Non-critical issue |
| `[fail]` | Red | Critical problem |
| `[fixed]` | Cyan | Auto-remediated |

Checks performed:

1. **Config directory** -- verifies the instance's config directory
   (`~/.config/mosaico/` by default) exists; creates it if missing
2. **config.toml** -- validates TOML syntax via `ConfigStore::try_load_config()`
3. **Theme** -- reports the theme in use, such as `[ok] Theme: tokyo-night night`,
   or warns that the `[theme]` name or flavor is unknown and names what it
   falls back to; it asks `ThemeConfig::check()`, which the daemon's
   `resolve()` is built on, so both agree
4. **keybindings.toml** -- validates syntax via `try_load_keybindings()`
5. **Keybinding keys** -- verifies each key name is one mosaico knows, with
   the check the daemon uses (`Keybinding::combo()` in core); reports the
   total count and any unknown key names
6. **rules.toml** -- validates syntax via `try_load_rules()`
7. **bar.toml** -- validates syntax via `try_load_bar()`, then, on the next
   line, says how many click bindings the bar has, counted from the
   bindings core resolves for it (`WidgetClicks`), or that it takes no
   clicks while the bar or `[mouse] enabled` is off, and flags each
   `[[mouse.click]]` entry the bar leaves out, and why, with the check the
   bar manager logs them with (`WidgetClicks::rejected()`). An entry with
   `action = "none"` removes a binding and is not flagged. Both checks are
   in `doctor_bar.rs`
8. **Autostart** -- compares what is registered to start at login with what
   `mosaico autostart enable` would register from the running executable
   (`AutostartStatus::read` in `mosaico_core::system`): `[ok]` when they
   match, `[warn]` when nothing is registered, when the user turned the
   entry off (disabled the logon task, or, for an older Run entry, turned it
   off in the startup apps settings), or
   when another command is registered (another copy of mosaico, or other
   arguments, such as the `start` without `--quiet` older releases
   registered), `[fail]` when the registered program is
   not a file (the binary moved or was deleted), the entry names no program,
   or the registration cannot be read. A program on a network share is not
   looked at, so it counts as another command. Doctor only reports; it never
   changes the registration
9. **Daemon** -- checks IPC pipe, PID file, and process liveness; cleans up
   stale PID files
10. **Monitors** -- enumerates monitors and reports count with per-monitor
   dimensions and positions
11. **Window transitions** -- reports whether windows move to their new place
   in transitions, then the `[animations]` settings the daemon starts with
   (`enabled`, `duration`, `easing`, `follow_system`; the defaults when
   `config.toml` is missing or invalid, said to be the defaults for an
   invalid file), whether the system lets windows be animated now
   (`Desktop::animations_allowed`), and whether this desktop can move windows
   frame by frame (`place_frames` with no frames, which moves nothing,
   returns `Ok` or `Error::Unsupported`). `[warn]` when transitions are
   enabled but windows snap anyway, naming the first reason: a duration of
   0, the system's setting while `follow_system` is on, or a desktop without
   frames (`Error::Unsupported`). When the desktop cannot be asked about
   frames, for any other error, `[fail]` with the error, and a `[warn]` that
   whether windows can move frame by frame is unknown

### `mosaico autostart enable|disable|status`

Manages starting mosaico at login, through the platform's `Autostart`
(`mosaico_core::system`):

- `enable` registers the running executable (`current_exe`) to run
  `mosaico start --quiet` at login, replacing any earlier registration and
  turning it back on if the user turned it off
- `disable` removes the registration
- `status` compares what is registered with what `enable` would register
  (`AutostartStatus::read`) and says whether autostart is enabled,
  disabled, turned off, or registered with an
  empty entry, another command, or a program that is not a file

Each prints what it did or found. One that fails prints `Error: <reason>`
and exits with status 1.

On Windows (`LogonTaskAutostart` in `mosaico-windows`) the registration is a
task named `Mosaico autostart for <user SID>` in the root folder of the Task
Scheduler Library, registered through its COM API. It starts at the user's
logon, as that user, without elevation, at priority 4 (normal; the Task
Scheduler's default is below normal), with no time limit and no delay.
Startup programs in the Run registry key start later: Explorer waits before
running them, then starts them one at a time. Disabling the task in the
Task Scheduler counts as turned off.

Older releases wrote a `Mosaico` value under
`HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`. While no
task exists that value, and its Startup apps switch, are still read as the
registration, so `status` and `doctor` report it as another command
(`start` without `--quiet`). `enable` and `disable` delete it.

### `mosaico action <verb> [direction]`

Sends a tiling action to the daemon over IPC. Actions now use a two-level
subcommand structure for directional actions:

```
mosaico action focus left
mosaico action focus right
mosaico action focus up
mosaico action focus down
mosaico action move left
mosaico action move right
mosaico action move up
mosaico action move down
mosaico action retile
mosaico action toggle-monocle
mosaico action toggle-float
mosaico action cycle-float
mosaico action grow-width
mosaico action shrink-height
mosaico action close-focused
mosaico action cycle-layout
mosaico action go-to-workspace 3
mosaico action go-to-workspace-next
mosaico action go-to-workspace-prev
mosaico action send-to-workspace 5
```

The workspace number is an argument here. Key bindings name the same
actions with the number in the name instead: `goto-workspace-N` and
`send-to-workspace-N` (see [actions.md](actions.md#serialization)). The steps
to the next and previous workspace also go by the names key bindings use,
`goto-workspace-next` and `goto-workspace-prev`. They act on the focused
monitor and follow `[workspaces] wrap`.

It exits with status 0 when the daemon carried the action out, and also when
there was nothing to act on (focusing left at the left edge, closing with
nothing focused), so scripts can send actions without checking the desktop
first. It exits with status 1 and prints why when the daemon could not carry
the action out, such as a `retile` that could not read the monitors.

### `mosaico update [--force]`

Checks GitHub for a newer release. If one is available, stops the running daemon
(if any), downloads the release zip, extracts the new binary, replaces the
current executable using a rename-then-write strategy, and restarts the daemon if
it was running before.

The `--force` flag reinstalls even if already on the latest version, useful for
recovering a corrupted binary or verifying the update mechanism.

The command itself is the same on every platform. Which release archive it
downloads, which file in it is the binary, and how that binary replaces the
running one come from the platform's `Installer` (`mosaico_core::update`). On a
platform that cannot update itself, it says so before it stops the daemon or
downloads anything.

On Windows the archive is the one for the CPU the build is for:
`mosaico-windows-amd64.zip` for x86-64, the only one published so far, or
`mosaico-windows-arm64.zip` for ARM64. The self-replacement
works by renaming the running `mosaico.exe` to `mosaico.exe.old` (Windows allows
renaming but not overwriting a locked file), then writing the new binary in its
place. The `.old` backup is cleaned up immediately or on the next daemon start.

### `mosaico pause`

Pauses all mosaico global hotkeys by unregistering them from the system. While
paused, mosaico's key combinations are fully released to the OS and can be used
by other applications. The `toggle-pause` hotkey (if configured) remains
registered so you can resume without a terminal.

```sh
mosaico pause
```

### `mosaico unpause`

Re-registers all hotkeys that were paused. Mosaico resumes normal hotkey
handling immediately.

```sh
mosaico unpause
```

### `mosaico banner`

Plays the animated logo that `start` and `doctor` show, on its own, and
exits with status 0.

### `mosaico daemon` (hidden)

Runs the daemon in-process. This subcommand is hidden from `--help` output
because users should use `mosaico start` instead. The `start` command
re-spawns the binary with this subcommand as a detached process.

It runs `mosaico_core::lifecycle::run()` (see [daemon.md](daemon.md)) and
exits with status 1 after printing `Daemon error: <reason>` when the daemon
cannot run, for example `Daemon error: Mosaico daemon is already running.`
when another daemon of the instance holds the lock.

### `mosaico debug list`

Lists every window the platform's `Enumerator` reports, showing: window id
(hex), title, class, executable path, managed status, monitor number, state,
position, dimensions, and border offset. Then lists the monitors left to right
with their keys. Exits with status 1 when the windows cannot be listed.

### `mosaico debug events`

Streams real-time window events to stdout. Starts the window event source
without hotkeys, prints each event as it arrives. Ctrl+C gracefully stops the
event source.

### `mosaico debug move --window <id> --x <x> --y <y> --w <width> --h <height>`

Moves a specific window to exact pixel coordinates with the same placement
call the daemon uses, but without skipping a window that is already in place
or repainting it afterwards. Takes the window id `debug list` shows (`0x`
hex), or the same number in decimal; `--hwnd` is still accepted for
`--window`. Exits with status 1 when the id is not a window id or the window
cannot be moved.

### `mosaico debug state`

Asks the daemon for its view of the desktop (the `Inspect` command) and
prints it: each monitor with its key, work area and active workspace, the
windows on each workspace that has any, the disconnected monitors and how
many windows were displaced from them, whether the daemon runs elevated,
and what has focus. The daemon's own text is printed when it sends one;
otherwise the `DesktopSnapshot` it sends is rendered. Exits with status 1,
saying why, when the request fails, for example when no daemon is running.

## Instances

Everything that identifies a daemon is derived from its instance
(`mosaico_core::Instance`): the IPC pipe, the single instance mutex, the PID
file, and the config and log directories. Without any of the variables below
the CLI and daemon use the default instance, with the usual names and paths.

| Variable | Effect |
|----------|--------|
| `MOSAICO_INSTANCE` | Selects a named instance. Pipe `\\.\pipe\mosaico-<name>`, mutex `Global\MosaicoDaemon-<name>`, files under `instances\<name>` in the usual config and data folders. 1-64 ASCII letters, digits, `-` or `_`. |
| `MOSAICO_CONFIG_DIR` | Overrides the config directory (config files and logs). Used as given. |
| `MOSAICO_DATA_DIR` | Overrides the data directory (PID file). Used as given. |

The usual folders come from the platform (`mosaico_core::user_dirs::UserDirs`).
On Windows the config directory is `.config\mosaico` in the profile folder
(`USERPROFILE`, or `HOME` when `USERPROFILE` is not set) and the data directory
is `mosaico` in `LOCALAPPDATA`. When the variable a folder comes from is not
set, is empty or is not an absolute path, and the override is not set either,
every command exits with status 2 and says which variable is wrong and which
override to set, for example `could not determine the config directory:
USERPROFILE is empty; set MOSAICO_CONFIG_DIR to choose one`. The overrides
themselves are used as given, relative paths included.

`mosaico start` passes its environment to the daemon it launches, so both
belong to the same instance. An invalid instance name makes every command
exit with status 2 rather than fall back to the default instance. The
integration tests use this to run each test against a private instance.

## Design Decisions

- The `daemon` subcommand is hidden so users never invoke it directly.
  `start` handles the process lifecycle.
- `start` uses Windows-specific detach flags rather than a service manager,
  keeping the implementation simple and dependency-free.
- The `banner` module draws the logo `start` and `doctor` share, and
  `mosaico banner` plays it on its own.
- Action verbs use a nested subcommand pattern (`action focus left`) rather
  than flat kebab-case (`action focus-left`) for a clearer CLI experience.
- Commands that show or move windows run outside the daemon process, so
  `main` sets the platform up once per process (`platform::init()`, which on
  Windows makes the process per-monitor DPI aware) before any command runs.
  The daemon, the debug commands and `doctor` all see windows and monitors in
  the same coordinates.

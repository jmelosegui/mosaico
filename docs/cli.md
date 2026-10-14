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
| `crates/mosaico/src/commands/mod.rs` | Module declarations for all commands |
| `crates/mosaico/src/commands/init.rs` | `mosaico init` handler |
| `crates/mosaico/src/commands/start.rs` | `mosaico start` handler |
| `crates/mosaico/src/commands/stop.rs` | `mosaico stop` handler |
| `crates/mosaico/src/commands/status.rs` | `mosaico status` handler |
| `crates/mosaico/src/commands/doctor.rs` | `mosaico doctor` handler |
| `crates/mosaico/src/commands/daemon.rs` | `mosaico daemon` handler (hidden) |
| `crates/mosaico/src/commands/action.rs` | `mosaico action <verb>` handler |
| `crates/mosaico/src/commands/pause.rs` | `mosaico pause` / `mosaico unpause` handler |
| `crates/mosaico/src/commands/banner.rs` | Shared ASCII logo used by `start` and `doctor` |
| `crates/mosaico/src/commands/update.rs` | `mosaico update` handler |
| `crates/mosaico/src/commands/debug/list.rs` | `mosaico debug list` handler |
| `crates/mosaico/src/commands/debug/events.rs` | `mosaico debug events` handler |
| `crates/mosaico/src/commands/debug/move_window.rs` | `mosaico debug move` handler |

### Command Context

Commands do not reach for the OS themselves. They take a
`commands::context::Context` holding everything they use: the daemon (through
`mosaico_core::ipc::IpcClient`), process control
(`mosaico_core::system::ProcessControl`), start at login
(`mosaico_core::system::Autostart`), window control
(`mosaico_core::WindowControl`), the windows and monitors on the desktop
(`mosaico_core::tiler::Enumerator`), the path of the running executable
(`current_exe`, which is what autostart registers), file checks that never
wait on the network (`mosaico_core::system::FileProbe`, for the autostart
check), the instance's `PidFile`, a clock, the update check, the logo, and
the output and error streams. They return an `Exit` (`Success` or
`Failure`), and only `main` calls `std::process::exit`.

`crates/mosaico/src/platform.rs` builds the real context; it is the one place
in the CLI that picks a platform implementation. Commands that move windows
or show where windows and monitors are run through `platform::run_on_desktop`,
which first makes the process DPI aware so coordinates match the daemon's.
The unit tests build one from `commands/test_support.rs`, a fake `World`
standing in for the daemon, processes, registry, desktop and clock, and
`commands/fake_desktop.rs`, a fake list of windows and monitors, so `start`,
`stop`, `status`, `autostart`, `action`, `pause`, `debug move`, `debug list`
and `doctor`'s autostart, daemon and monitor checks are tested without a
daemon or a desktop.

Still calling the platform crate directly: `daemon` (it is the daemon),
`debug events`' Ctrl+C handler and window titles, and `update`'s version
lookup and download, which wait for the signal (#48) and HTTP (#54)
contracts. `debug events` gets its event source from
`platform::event_source()`.

### Key Types

- `Cli` -- top-level `clap::Parser` struct
- `Commands` -- enum of all subcommands: `Init`, `Start`, `Stop`, `Status`,
  `Doctor`, `Update`, `Action`, `Pause`, `Unpause`, `Debug`, `Daemon`
- `ActionCommands` -- enum: `Focus { direction }`, `Move { direction }`,
  `Retile`, `ToggleMonocle`, `CloseFocused`, `GoToWorkspace { n }`,
  `SendToWorkspace { n }`
- `DirectionCommands` -- enum: `Left`, `Right`, `Up`, `Down`
- `DebugCommands` -- enum for debug sub-subcommands: `List`, `Events`, `Move`

## Available Commands

### `mosaico init`

Creates default configuration files in `~/.config/mosaico/`:

- `config.toml` -- layout, border, and logging settings
- `keybindings.toml` -- keyboard shortcuts
- `rules.toml` -- window management rules

Skips files that already exist to avoid overwriting user customizations.

### `mosaico start`

Launches the daemon as a detached background process. Re-executes the current
binary with the hidden `daemon` subcommand using Windows process creation flags
`CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW` (constant `DETACH_FLAGS =
0x08000200`) to fully detach from the terminal.

On success, prints a startup banner showing the ASCII logo, config directory,
PID, repository URL, and a rotating tip. Tips cycle based on the current
timestamp so users see a different one each launch. After the banner, a
version check queries the GitHub releases API (via `version_check` module)
and prints an update notice if a newer release exists.

If a daemon is already running (detected via IPC pipe check), it reports the
existing state instead of starting a second instance.

### `mosaico stop`

Sends a `Stop` command to the running daemon over IPC (named pipe). If the
IPC pipe is unavailable (e.g. the IPC thread crashed), falls back to reading
the PID file and terminating the process via `kill_process()`. Reports
success or failure.

### `mosaico status`

Checks whether the daemon is running by:

1. Reading the PID file
2. Verifying the process is alive via `OpenProcess`
3. Checking if the IPC pipe exists

Reports one of: running (with PID), stale PID (dead process), or not running.

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

1. **Config directory** -- verifies `~/.config/mosaico/` exists; creates it
   if missing
2. **config.toml** -- validates TOML syntax via `try_load()`
3. **keybindings.toml** -- validates syntax via `try_load_keybindings()`
4. **Keybinding keys** -- verifies each key name is one mosaico knows, with
   the check the daemon uses (`Keybinding::combo()` in core); reports the
   total count and any unknown key names
5. **rules.toml** -- validates syntax via `try_load_rules()`
6. **bar.toml** -- validates syntax via `try_load_bar()`
7. **Autostart** -- compares what is registered to start at login with what
   `mosaico autostart enable` would register from the running executable
   (`AutostartStatus::of` in `mosaico_core::system`): `[ok]` when they match,
   `[warn]` when nothing is registered or another command is (another copy
   of mosaico, or other arguments), `[fail]` when the registered program is
   not a file (the binary moved or was deleted), the entry names no program,
   or the registration cannot be read. A program on a network share is not
   looked at, so it counts as another command. Doctor only reports; it never
   changes the registration
8. **Daemon** -- checks IPC pipe, PID file, and process liveness; cleans up
   stale PID files
9. **Monitors** -- enumerates monitors and reports count with per-monitor
   dimensions and positions

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
mosaico action close-focused
mosaico action goto-workspace 3
mosaico action send-to-workspace 5
```

### `mosaico update [--force]`

Checks GitHub for a newer release. If one is available, stops the running daemon
(if any), downloads the release zip, extracts the new binary, replaces the
current executable using a rename-then-write strategy, and restarts the daemon if
it was running before.

The `--force` flag reinstalls even if already on the latest version, useful for
recovering a corrupted binary or verifying the update mechanism.

The self-replacement works by renaming the running `mosaico.exe` to
`mosaico.exe.old` (Windows allows renaming but not overwriting a locked file),
then writing the new binary in its place. The `.old` backup is cleaned up
immediately or on the next daemon start.

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

### `mosaico daemon` (hidden)

Runs the daemon in-process. This subcommand is hidden from `--help` output
because users should use `mosaico start` instead. The `start` command
re-spawns the binary with this subcommand as a detached process.

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

## Instances

Everything that identifies a daemon is derived from its instance
(`mosaico_core::Instance`): the IPC pipe, the single instance mutex, the PID
file, and the config and log directories. Without any of the variables below
the CLI and daemon use the default instance, with the usual names and paths.

| Variable | Effect |
|----------|--------|
| `MOSAICO_INSTANCE` | Selects a named instance. Pipe `\\.\pipe\mosaico-<name>`, mutex `Global\MosaicoDaemon-<name>`, files under `instances\<name>` in the usual config and data folders. 1-64 ASCII letters, digits, `-` or `_`. |
| `MOSAICO_CONFIG_DIR` | Overrides the config directory (config files and logs). |
| `MOSAICO_DATA_DIR` | Overrides the data directory (PID file). |

`mosaico start` passes its environment to the daemon it launches, so both
belong to the same instance. An invalid instance name makes every command
exit with status 2 rather than fall back to the default instance. The
integration tests use this to run each test against a private instance.

## Design Decisions

- The `daemon` subcommand is hidden so users never invoke it directly.
  `start` handles the process lifecycle.
- `start` uses Windows-specific detach flags rather than a service manager,
  keeping the implementation simple and dependency-free.
- The `banner` module is private -- only used internally by `start` and
  `doctor` for the shared ASCII logo.
- Action verbs use a nested subcommand pattern (`action focus left`) rather
  than flat kebab-case (`action focus-left`) for a clearer CLI experience.
- Commands that show or move windows run outside the daemon process, so they
  enable DPI awareness themselves, through `platform::run_on_desktop`
  (`debug list`, `debug move`). Other commands, `doctor` included, run without
  it.

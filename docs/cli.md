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
| `crates/mosaico/src/commands/daemon.rs` | `mosaico daemon` handler (hidden) |
| `crates/mosaico/src/commands/action.rs` | `mosaico action <verb>` handler |
| `crates/mosaico/src/commands/debug/list.rs` | `mosaico debug list` handler |
| `crates/mosaico/src/commands/debug/events.rs` | `mosaico debug events` handler |
| `crates/mosaico/src/commands/debug/move_window.rs` | `mosaico debug move` handler |

### Key Types

- `Cli` -- top-level `clap::Parser` struct
- `Commands` -- enum of all subcommands: `Init`, `Start`, `Stop`, `Status`,
  `Action`, `Debug`, `Daemon`
- `ActionCommands` -- enum mirroring each `Action` variant as a subcommand
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

On success, prints a colorful ANSI banner showing the Mosaico logo, PID, and
version.

If a daemon is already running (detected via the PID file), it reports the
existing PID instead of starting a second instance.

### `mosaico stop`

Sends a `Stop` command to the running daemon over IPC (named pipe). Reports
success or failure.

### `mosaico status`

Checks whether the daemon is running by:

1. Reading the PID file
2. Verifying the process is alive via `OpenProcess`
3. Checking if the IPC pipe exists

Reports one of: running (with PID), stale PID (dead process), or not running.

### `mosaico action <verb>`

Sends a tiling action to the daemon over IPC. Available verbs map directly to
`Action` enum variants:

```
mosaico action focus-next
mosaico action focus-prev
mosaico action swap-next
mosaico action swap-prev
mosaico action retile
mosaico action focus-monitor-next
mosaico action focus-monitor-prev
mosaico action move-to-monitor-next
mosaico action move-to-monitor-prev
mosaico action toggle-monocle
mosaico action close-focused
```

### `mosaico daemon` (hidden)

Runs the daemon in-process. This subcommand is hidden from `--help` output
because users should use `mosaico start` instead. The `start` command
re-spawns the binary with this subcommand as a detached process.

### `mosaico debug list`

Displays a formatted table of all visible windows showing: HWND (hex),
managed status, monitor number, title, class, position, dimensions, and border
offset. Uses `comfy-table` for formatting.

### `mosaico debug events`

Streams real-time window events to stdout. Starts the Win32 event loop without
hotkeys, prints each event as it arrives. Ctrl+C gracefully stops the event
loop.

### `mosaico debug move <hwnd> <x> <y> <width> <height>`

Moves a specific window by its HWND to exact pixel coordinates. Accepts HWND
in decimal or `0x` hex format.

## Design Decisions

- The `daemon` subcommand is hidden so users never invoke it directly.
  `start` handles the process lifecycle.
- `start` uses Windows-specific detach flags rather than a service manager,
  keeping the implementation simple and dependency-free.
- All action verbs use kebab-case (`focus-next`, not `focusNext`) matching
  the serde serialization of the `Action` enum.
- Debug commands enable DPI awareness independently since they run outside the
  daemon process.

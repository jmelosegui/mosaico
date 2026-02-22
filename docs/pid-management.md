# PID File Management

Mosaico uses a PID (Process ID) file to track the running daemon process.
This enables detection of running, stopped, or stale daemon states across
CLI commands.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/pid.rs` | `pid_path()`, `write_pid_file()`, `read_pid_file()`, `remove_pid_file()` |
| `crates/mosaico-windows/src/process.rs` | `is_process_alive()` |

### Key Types

- All functions are free-standing; no structs are defined.

## PID File Location

The PID file is stored at `%LOCALAPPDATA%\mosaico\mosaico.pid`. The data
directory (`%LOCALAPPDATA%\mosaico\`) is created automatically if it does
not exist.

The `data_dir()` function resolves this path using `dirs::data_local_dir()`.

## Functions

### `write_pid_file()`

Writes the current process PID as a string to the PID file. Called at daemon
startup in `daemon::run()`.

### `read_pid_file() -> Option<u32>`

Reads and parses the PID from the file. Returns `None` if the file does not
exist, is empty, or contains an invalid number.

### `remove_pid_file()`

Deletes the PID file. Called during daemon shutdown in `daemon::run()` after
the main loop exits.

### `is_process_alive(pid: u32) -> bool`

Windows-specific function that calls `OpenProcess` with
`PROCESS_QUERY_LIMITED_INFORMATION` to check if a process with the given PID
exists. Returns `true` if the process is alive, `false` otherwise.

This uses the minimal permission flag to check process existence without
requiring elevated privileges.

## Usage Across Commands

### `mosaico start`

1. Reads the PID file
2. If a PID exists and the process is alive: reports the daemon is already
   running with that PID
3. If a PID exists but the process is dead (stale): removes the stale PID
   file and starts a new daemon
4. If no PID file exists: starts the daemon normally

### `mosaico stop`

1. Sends a `Stop` command via IPC
2. The daemon's main loop exits, triggering PID file removal in the cleanup
   path

### `mosaico status`

1. Reads the PID file
2. If a PID exists and the process is alive: reports "running" with PID
3. If a PID exists but the process is dead: reports "stale PID" (crashed or
   killed without cleanup)
4. If no PID file exists: reports "not running"

### `mosaico daemon`

1. Calls `write_pid_file()` at the start of `daemon::run()`
2. Runs the daemon loop
3. Calls `remove_pid_file()` in the cleanup path (after the loop exits)

## Stale PID Handling

A stale PID file occurs when the daemon crashes or is killed without going
through the normal shutdown path (e.g., `taskkill /F`). The `start` and
`status` commands detect this by:

1. Reading the PID from the file
2. Checking if the process is alive via `is_process_alive()`
3. If dead: the PID file is considered stale

The `start` command automatically cleans up stale PID files before launching
a new daemon instance.

## Design Decisions

- **PID file in `%LOCALAPPDATA%`** rather than `~/.config/mosaico/` separates
  runtime state from user configuration. Configuration is in the config
  directory; runtime artifacts are in the local data directory.
- **Simple text format** (just the PID number as a string) is easy to read,
  write, and debug manually.
- **`PROCESS_QUERY_LIMITED_INFORMATION`** is the minimal permission needed to
  check if a process exists, avoiding unnecessary privilege requirements.
- **Automatic stale cleanup** in `start` means users never need to manually
  delete a PID file after a crash.
- **PID file is removed in the cleanup path of `daemon::run()`**, which
  uses a simple sequential flow (not a `Drop` guard), so cleanup runs even
  if the daemon loop returns an error.

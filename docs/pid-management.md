# PID File Management

Mosaico uses a PID (Process ID) file to track the running daemon process.
This enables detection of running, stopped, or stale daemon states across
CLI commands.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/pid.rs` | `PidFile` with `write()`, `read()` and `remove()` |
| `crates/mosaico-core/src/instance.rs` | `Instance::pid_path()`, where an instance keeps its PID file |
| `crates/mosaico-core/src/lifecycle.rs` | writes and removes the daemon's PID file |
| `crates/mosaico-core/src/system.rs` | `ProcessControl`, what commands use to check and end processes |
| `crates/mosaico-windows/src/process.rs` | `WindowsProcesses`, the Windows `ProcessControl` |
| `crates/mosaico/src/commands/stop.rs` | `mosaico stop` with PID-based fallback |

### Key Types

- `PidFile` -- the PID file of an instance, at `Instance::pid_path()`.

## PID File Location

The PID file is `mosaico.pid` in the instance's data directory:
`%LOCALAPPDATA%\mosaico\mosaico.pid` for the default instance. The directory
is created automatically if it does not exist. See Instances in
[cli.md](cli.md) for named instances and `MOSAICO_DATA_DIR`.

## Functions

### `PidFile::write(pid: u32)`

Writes the PID as a string to the PID file, creating its directory if
needed. `lifecycle::run()` calls it with the daemon's own PID once the
daemon holds the single instance lock.

### `PidFile::read() -> Result<Option<u32>>`

Reads and parses the PID from the file. Returns `Ok(None)` if the file does
not exist. A file that cannot be read is an `Error::Io`, and one that does not
hold a number an `Error::Io` of kind `InvalidData`; `start`, `stop` and
`status` treat both like a missing file.

### `PidFile::remove()`

Deletes the PID file, if there is one. `lifecycle::run()` calls it during
daemon shutdown, after the main loop exits, or while a panic in it unwinds,
and before it releases the single instance lock, so it never removes the PID
file of a daemon that starts next.

## Process Control

Commands check and end processes only through `ProcessControl`
(`cx.processes`); the Windows functions behind it are private to
`process.rs`.

### `ProcessControl::is_alive(pid: u32) -> bool`

On Windows, calls `OpenProcess` with `PROCESS_QUERY_LIMITED_INFORMATION` to
check if a process with the given PID exists. Returns `true` if the process
is alive, `false` otherwise.

This uses the minimal permission flag to check process existence without
requiring elevated privileges.

### `ProcessControl::kill(pid: u32) -> Result<()>`

On Windows, opens the process with `PROCESS_TERMINATE` permission and calls
`TerminateProcess` to forcefully kill it. Fails with `Error::Platform` when
the process cannot be opened or terminated, which includes a process that
exited after `mosaico stop` found it alive. Used as a fallback when the IPC
pipe is unavailable. When it fails, `mosaico stop` reports `Failed to kill
process <pid>.` and keeps the PID file, so the process can still be found.

## Usage Across Commands

### `mosaico start`

1. Reads the PID file
2. If a PID exists and the process is alive: reports the daemon is already
   running with that PID
3. If a PID exists but the process is dead (stale): removes the stale PID
   file and starts a new daemon
4. If no PID file exists: starts the daemon normally

### `mosaico stop`

1. Attempts graceful shutdown by sending a `Stop` command via IPC
2. If IPC succeeds: the daemon's main loop exits, and PID file is removed by
   both the daemon cleanup path and the stop command
3. If IPC fails (pipe unavailable): falls back to reading the PID file and
   calling `ProcessControl::kill(pid)` to forcefully terminate the daemon
   process, then removes the PID file

### `mosaico status`

1. Reads the PID file
2. If a PID exists and the process is alive: reports "running" with PID
3. If a PID exists but the process is dead: reports "stale PID" (crashed or
   killed without cleanup)
4. If no PID file exists: reports "not running"

### `mosaico daemon`

1. `lifecycle::run()` writes the PID file once the daemon holds the single
   instance lock; a daemon refused by the lock leaves the running daemon's
   PID file alone
2. Runs the daemon loop
3. Removes the PID file in the cleanup path (after the loop exits), then
   releases the lock

## Stale PID Handling

A stale PID file occurs when the daemon crashes or is killed without going
through the normal shutdown path (e.g., `taskkill /F`). The `start` and
`status` commands detect this by:

1. Reading the PID from the file
2. Checking if the process is alive via `ProcessControl::is_alive()`
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
- **PID file is removed in the cleanup path of `lifecycle::run()`**, which
  keeps it in a guard dropped before the lock, so it is removed before the
  lock is released when the daemon loop returns, with or without an error,
  and when a panic in it unwinds. Release builds abort on a panic and leave
  the PID file behind, as a crash does, which `start` and `status` treat as
  stale.
- **PID-based kill fallback** in `stop` ensures the daemon can be stopped
  even when the IPC thread has crashed, covering scenarios like thread panics
  or resource exhaustion that leave the process alive but unresponsive.
- **Killing uses `PROCESS_TERMINATE`** on Windows rather than sending a signal
  because Windows does not have Unix-style signals; `TerminateProcess` is
  the standard forceful shutdown mechanism.

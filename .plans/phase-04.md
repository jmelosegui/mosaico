# Phase 4: Daemon + IPC

**Status:** Complete

**Goal:** Make `mosaico start` launch a long-running background process (daemon)
that stays alive and manages windows. `mosaico stop` signals the daemon to shut
down. `mosaico status` reports whether the daemon is running.

## Why a daemon?

A tiling window manager needs a persistent process that:

- Listens for window events (open, close, move, resize, focus)
- Reacts by tiling/repositioning windows
- Accepts commands from the CLI (e.g. "focus left", "move to workspace 2")

The CLI (`mosaico`) acts as a thin client that sends commands to the daemon. This
is a common architecture used by most tiling window managers.

## IPC mechanism: Named Pipes

On Windows, Named Pipes are the standard IPC mechanism. They work like a local
socket — the daemon creates a pipe server, and the CLI connects as a client to
send commands and receive responses.

Named pipe path: `\\.\pipe\mosaico`

## Target structure (new/modified files only)

```
crates/
  mosaico/
    src/
      commands/
        start.rs              # launch the daemon process
        stop.rs               # send stop command via IPC
        status.rs             # check if daemon is running
  mosaico-core/
    src/
      ipc.rs                  # IPC message types (Command, Response)
  mosaico-windows/
    src/
      ipc.rs                  # Named Pipe server + client implementation
      daemon.rs               # Main daemon loop
```

## Tasks

- [x] Define IPC message types in `mosaico-core` (Command enum, Response enum)
- [x] Implement Named Pipe server in `mosaico-windows` (daemon side)
- [x] Implement Named Pipe client in `mosaico-windows` (CLI side)
- [x] Implement daemon main loop in `mosaico-windows/src/daemon.rs`
- [x] Update `start` command to launch the daemon
- [x] Update `stop` command to send a shutdown message via IPC
- [x] Update `status` command to check if the daemon pipe exists
- [x] Write a PID file to `%LOCALAPPDATA%\mosaico\mosaico.pid` for process tracking
- [x] Add integration tests
- [x] Commit

## IPC protocol sketch

Commands are sent as newline-delimited JSON over the named pipe:

```
CLI -> Daemon:  {"command": "Stop"}
Daemon -> CLI:  {"status": "ok"}

CLI -> Daemon:  {"command": "Status"}
Daemon -> CLI:  {"status": "ok", "uptime": 3600}
```

## Rust learning notes

- `std::process::Command` — spawn a child process (for starting the daemon).
  Similar to `Process.Start()` in C#.
- `serde` + `serde_json` — serialization/deserialization. The Rust equivalent of
  `System.Text.Json`. Derive `Serialize` and `Deserialize` on your types and
  they can be converted to/from JSON automatically.
- `std::io::BufReader` / `BufWriter` — buffered I/O for reading/writing pipes.
  Similar to `StreamReader` / `StreamWriter` in C#.
- Named Pipes on Windows — `CreateNamedPipeW` (server) and `CreateFileW` (client)
  from the Win32 API, or the `interprocess` crate for a cross-platform abstraction.

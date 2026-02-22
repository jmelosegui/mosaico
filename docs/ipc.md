# Inter-Process Communication (IPC)

The daemon and CLI communicate via a Windows Named Pipe. The CLI connects as a
client, sends a JSON-serialized command, and reads back a JSON-serialized
response. Each connection handles one request-response cycle.

## Architecture

The IPC system spans two crates:

- **mosaico-core** defines platform-agnostic message types (`Command`,
  `Response`)
- **mosaico-windows** implements the Win32 named pipe server and client

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/ipc.rs` | `Command`, `Response`, `ResponseStatus`, `PIPE_NAME` |
| `crates/mosaico-windows/src/ipc.rs` | `PipeServer`, `send_command()`, `is_daemon_running()` |

### Key Types

- `PIPE_NAME` -- constant `r"\\.\pipe\mosaico"`
- `Command` (enum, tagged JSON via `#[serde(tag = "command")]`):
  - `Stop` -- request daemon shutdown
  - `Status` -- health check
  - `Action { action: Action }` -- forward a tiling action
- `Response` -- fields: `status: ResponseStatus`, `message: Option<String>`
  - Factory methods: `ok()`, `ok_with_message(msg)`, `error(msg)`
- `ResponseStatus` (enum) -- `Ok`, `Error`
- `PipeServer` (struct) -- server-side pipe wrapper with RAII cleanup
- `HandleGuard` (struct) -- RAII wrapper that closes a HANDLE on drop

## Protocol

Messages are JSON objects terminated by a newline character (`\n`):

### Request Format

```json
{"command":"action","action":"focus-next"}
```

```json
{"command":"stop"}
```

### Response Format

```json
{"status":"ok"}
```

```json
{"status":"ok","message":"Daemon is running"}
```

```json
{"status":"error","message":"Unknown command"}
```

## Server Side

The daemon's `ipc_loop()` runs on a dedicated thread:

1. Creates a `PipeServer` via `CreateNamedPipeW` with:
   - `PIPE_ACCESS_DUPLEX` -- bidirectional communication
   - `PIPE_TYPE_BYTE | PIPE_READMODE_BYTE` -- byte-stream mode
   - `PIPE_UNLIMITED_INSTANCES` -- allows multiple pipe instances
2. Blocks on `ConnectNamedPipe` waiting for a client
3. Reads the command JSON line via `accept_command()`
4. Deserializes the `Command`
5. Sends `DaemonMsg::Command(cmd, reply_tx)` to the main daemon thread
6. Waits for the reply on the oneshot `reply_rx` channel
7. Serializes and sends the `Response` via `send_response()`
8. Calls `FlushFileBuffers` then `DisconnectNamedPipe`
9. Loops back to wait for the next client

### Handle Management

`PipeServer` uses `DuplicateHandle` to create independent `std::fs::File`
wrappers from the pipe HANDLE. This avoids double-close bugs -- the `File`
owns a duplicate handle, so dropping the `File` does not close the original
pipe handle.

`Drop` implementation on `PipeServer` calls `CloseHandle` on the original
pipe handle.

## Client Side

`send_command(command)`:

1. Opens the pipe with `CreateFileW` using `GENERIC_READ | GENERIC_WRITE`
2. Wraps the handle as a `File` via `duplicate_handle_as_file()`
3. Writes the JSON-serialized `Command` followed by `\n`
4. Reads the response line
5. Deserializes and returns the `Response`
6. Handle is automatically closed when `HandleGuard` drops

## Daemon Detection

`is_daemon_running()`:

- Uses `WaitNamedPipeW` with a 1ms timeout to check if the pipe exists
- Returns `true` if the pipe is reachable, `false` otherwise
- Does not consume a connection -- safe for status checks
- Used by `start`, `stop`, `status`, and `action` CLI commands

## Integration

The IPC system connects the CLI commands to the daemon:

```
CLI (mosaico action focus-next)
  -> send_command(Command::Action { action: FocusNext })
  -> Named pipe \\.\pipe\mosaico
  -> ipc_loop() receives command
  -> DaemonMsg::Command forwarded to main thread
  -> TilingManager processes action
  -> Response sent back through pipe
  -> CLI displays result
```

## Design Decisions

- **JSON-over-pipe with newline delimiters** keeps the protocol simple and
  debuggable. Each message is one line.
- **Tagged union serialization** (`#[serde(tag = "command")]`) produces
  clean JSON: `{"command":"stop"}` rather than `{"Stop":{}}`.
- **`FlushFileBuffers` before `DisconnectNamedPipe`** prevents error 233
  (client reads before server data arrives in the pipe buffer).
- **`DuplicateHandle`** for `File` creation avoids double-close bugs that
  would occur if both `File::drop()` and `PipeServer::drop()` closed the
  same handle.
- **`PIPE_UNLIMITED_INSTANCES`** allows the pipe name to be reused across
  disconnect/reconnect cycles, though only one `ipc_loop` thread runs at a
  time.
- **1ms `WaitNamedPipe` timeout** for daemon detection is fast enough for
  CLI responsiveness without blocking.

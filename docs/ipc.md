# Inter-Process Communication (IPC)

The CLI and the daemon talk over a local channel: the CLI connects, sends one
request, reads one reply and disconnects. On Windows the channel is a named
pipe.

## Architecture

The protocol lives in core and the platform only supplies byte streams:

- **mosaico-core** (`crates/mosaico-core/src/ipc/`) defines the messages, the
  framing, the protocol version, the transport traits and the daemon's accept
  loop.
- **mosaico-windows** (`crates/mosaico-windows/src/ipc.rs`) implements the
  traits with named pipes.
- **mosaico** (the CLI) picks the implementation in one place,
  `crates/mosaico/src/platform.rs`, and every command goes through the
  `IpcClient` trait.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/ipc/mod.rs` | `Command`, `PROTOCOL_VERSION`, `DAEMON_VERSION` |
| `crates/mosaico-core/src/ipc/request.rs` | `Request` envelope and `decode_request()` |
| `crates/mosaico-core/src/ipc/response.rs` | `Response`, `ErrorCode`, `DaemonStatus` |
| `crates/mosaico-core/src/ipc/framing.rs` | `read_message()`, `write_message()`, `MAX_MESSAGE_BYTES` |
| `crates/mosaico-core/src/ipc/transport.rs` | `IpcClient`, `IpcServer`, `IpcConnection`, `serve()` |
| `crates/mosaico-windows/src/ipc.rs` | `PipeClient`, `PipeServer`, `PipeConnection`, `DEFAULT_PIPE_NAME` |
| `crates/mosaico-windows/src/daemon_ipc.rs` | Runs `serve()` on the daemon's IPC thread |

## Framing

Every message is one compact JSON value followed by `\n`. Compact JSON never
contains a raw newline, so the newline always ends the message. A message
larger than `MAX_MESSAGE_BYTES` (1 MiB) is rejected rather than buffered.

If the other side goes away before a whole message arrives, the reader gets
`IpcError::Closed`. That is what the CLI sees when the daemon exits
mid-request, instead of hanging.

## Requests

A request is the command plus the protocol version of the client:

```json
{"version":1,"command":"Stop"}
```

```json
{"version":1,"command":"Action","action":"focus-left"}
```

The version sits next to the command's own fields, so a daemon built before
versioning still reads the command and ignores the version.

The daemon checks every request before running it, and always answers:

| Request | Reply |
|---------|-------|
| not JSON, or not an object | `malformed` |
| larger than the limit | `too-large` |
| from another `PROTOCOL_VERSION`, or with no version | `version-mismatch`, with the daemon's `DaemonStatus` in `data` |
| `Stop` from any version | carried out, so a mismatched daemon can always be stopped |
| an unknown command | `unknown-command` |
| a known command with bad fields | `malformed` |

Bump `PROTOCOL_VERSION` whenever a change would make one side misread the
other.

## Responses

```json
{"status":"ok","message":"Daemon is running, managing 3 windows","data":{"version":"0.9.1","protocol":1,"windows":3}}
```

```json
{"status":"error","message":"...","code":"version-mismatch","data":{"version":"0.9.1","protocol":1,"windows":0}}
```

- `status` -- `ok` or `error`
- `message` -- text for people, optional
- `code` -- an `ErrorCode` for programs, on errors. Codes a build does not
  know read as `Other`.
- `data` -- a typed result, read with `Response::data::<T>()`. `Status`
  returns a `DaemonStatus`.

`code` and `data` are optional, so replies from older daemons still parse.

## Transport Contract

`IpcClient` (CLI side):

- built for one daemon instance; on Windows that is a pipe name
- `connect` fails with `IpcError::NotRunning` straight away when nothing is
  listening, and waits a short, bounded time when the daemon is busy with
  another client
- `is_daemon_running` checks for a listener without taking a connection
- `send` (provided by core) writes the request and reads the reply

`IpcServer` / `IpcConnection` (daemon side):

- `accept` blocks until a client connects; one request and one reply per
  connection
- `finish` must not return until the client can read the whole reply

`serve()` runs the loop: accept, read, refuse or hand the command to the
daemon, reply, repeat. It returns `Served::Stopped` after answering `Stop`,
`Served::Abandoned` if the daemon thread is gone, or `Served::Failed` if the
listener can no longer accept connections. In that last case the daemon shuts
down cleanly and restores every window, rather than keep tiling with no way to
stop it.

## Windows Implementation

- `PipeServer::accept` creates a fresh pipe instance with `CreateNamedPipeW`
  (`PIPE_ACCESS_DUPLEX`, byte mode, unlimited instances) and waits in
  `ConnectNamedPipe`. A client that connects between the two is reported as
  `ERROR_PIPE_CONNECTED`, which counts as connected.
- `PipeConnection::finish` calls `FlushFileBuffers` before
  `DisconnectNamedPipe`. Disconnecting first throws away the unread reply and
  the client gets error 233.
- `PipeClient::connect` opens the pipe with `CreateFileW`. `ERROR_FILE_NOT_FOUND`
  means no daemon. On `ERROR_PIPE_BUSY` it waits up to 2 seconds with
  `WaitNamedPipeW` and tries once more.
- `PipeClient::is_daemon_running` uses `WaitNamedPipeW` with a 1 ms timeout.
- The pipe name is a parameter. The daemon and the CLI use
  `DEFAULT_PIPE_NAME` (`\\.\pipe\mosaico`).

## Testing

Framing, request decoding, error replies and the accept loop are tested in
core against in-memory streams (`crates/mosaico-core/src/ipc/tests/`), so none
of it needs a daemon or a pipe.

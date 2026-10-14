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
- `is_daemon_running` reports whether the endpoint exists, without taking a
  connection; a daemon busy with another client is running
- callers about to send do not check `is_daemon_running` first: `send`
  reports `IpcError::NotRunning` itself, and a separate check could see a
  busy daemon as missing
- `send` (provided by core) writes the request and reads the reply, waiting at
  most `REPLY_TIMEOUT` (10 seconds); `send_with_timeout` takes another limit.
  A daemon that is alive but does not reply is reported as
  `IpcError::TimedOut` instead of hanging the CLI. The CLI says the daemon is
  not responding, and `mosaico stop` then ends it from its PID file
- `IpcStream::set_read_timeout` makes the client's reads fail with `TimedOut`
  once the limit has passed. A transport whose reads can block must implement
  it

`IpcServer` / `IpcConnection` (daemon side):

- `accept` blocks until a client connects; one request and one reply per
  connection
- while one connection is being answered, the endpoint keeps accepting the
  next client
- a client that leaves before it is handed out is reported as
  `IpcError::Closed`: that client is lost, the server keeps listening and
  `serve()` waits for the next one. Any other `accept` error is fatal
- `finish` must not return until the client can read the whole reply
- `set_read_timeout` makes reads fail with `TimedOut` once the timeout has
  passed, for the whole request. A transport whose reads can block must
  implement it
- `set_write_timeout` does the same for writing the reply and `finish`. A
  transport whose writes or `finish` can wait for the client must implement it

`serve()` gives every client `REQUEST_TIMEOUT` (5 seconds) to send its
request, and again to read the reply. A client that connects and never sends,
or sends and never reads, is dropped instead of holding up everyone after it.
Connections are answered one at a time.

`serve()` runs the loop: accept, read, refuse or hand the command to the
daemon, reply, repeat. It returns `Served::Stopped` after answering `Stop`,
`Served::Abandoned` if the daemon thread is gone, or `Served::Failed` if the
listener can no longer accept connections. In that last case the daemon shuts
down cleanly and restores every window, rather than keep tiling with no way to
stop it.

## Windows Implementation

- `PipeServer::accept` waits in `ConnectNamedPipe` on a pipe instance created
  with `CreateNamedPipeW` (`PIPE_ACCESS_DUPLEX`, byte mode, unlimited
  instances). As soon as a client connects it creates the next instance, so
  while that client is answered a second one finds a free instance instead of
  `ERROR_PIPE_BUSY`. A client that connects between creation and
  `ConnectNamedPipe` is reported as `ERROR_PIPE_CONNECTED`, which counts as
  connected. Any other `ConnectNamedPipe` failure, such as `ERROR_NO_DATA` for
  a client that connected and closed its end first, concerns that client
  only: a fresh instance takes the failed one's place and `accept` returns
  `IpcError::Closed`. Only a failure of `CreateNamedPipeW` is fatal.
- Pipe handles are synchronous and have no timeouts of their own.
  `PipeConnection` enforces both deadlines with a watchdog thread
  (`ipc_watchdog.rs`) that calls `CancelSynchronousIo` on the serving thread
  if a `ReadFile`, `WriteFile` or `FlushFileBuffers` is still pending when the
  deadline passes. A reply bigger than the 4096 byte pipe buffer blocks in
  `WriteFile` until the client reads; a smaller one blocks in
  `FlushFileBuffers`.
- `PipeConnection::finish` calls `FlushFileBuffers` before
  `DisconnectNamedPipe`. Disconnecting first throws away the unread reply and
  the client gets error 233.
- `PipeClient::connect` opens the pipe with `CreateFileW`. `ERROR_FILE_NOT_FOUND`
  means no daemon. On `ERROR_PIPE_BUSY` it waits up to 2 seconds with
  `WaitNamedPipeW` and tries once more.
- `PipeClient::is_daemon_running` uses `WaitNamedPipeW` with a 1 ms timeout.
  Only `ERROR_FILE_NOT_FOUND` means no daemon; a timeout means every instance
  is busy and the daemon is running.
- `PipeClient::connect` returns a `PipeStream`, whose reads use the same
  watchdog as the server to enforce the reply timeout.
- The pipe name comes from the instance (`pipe_name`): `\\.\pipe\mosaico` for
  the default instance, `\\.\pipe\mosaico-<name>` for a named one.

## Testing

Framing, request decoding, error replies and the accept loop are tested in
core against in-memory streams (`crates/mosaico-core/src/ipc/tests/`), so none
of it needs a daemon or a pipe.

`crates/mosaico-windows/src/ipc_tests.rs` runs a real pipe server on a
unique pipe name and checks that a daemon busy with one client still answers
the next.

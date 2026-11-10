# Inter-Process Communication (IPC)

The CLI and the daemon talk over a local channel: the CLI connects, sends one
request, reads one reply and disconnects. On Windows the channel is a named
pipe.

Bars talk to the daemon over a second channel of their own, the bar API,
which keeps a session open as long as the bar runs (see [bar-api.md](bar-api.md)
and [The Bar API Pipe](#the-bar-api-pipe)).

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
| `crates/mosaico-core/src/snapshot.rs` | `DesktopSnapshot`, what `Inspect` returns (re-exported from `ipc`), and its `debug state` text |
| `crates/mosaico-core/src/ipc/framing.rs` | `read_message()`, `write_message()`, `MAX_MESSAGE_BYTES` |
| `crates/mosaico-core/src/ipc/transport.rs` | `IpcClient`, `IpcServer`, `IpcConnection`, `serve()` |
| `crates/mosaico-windows/src/ipc.rs` | `PipeClient`, `PipeServer`, `PipeConnection`, `DEFAULT_PIPE_NAME` |
| `crates/mosaico-windows/src/ipc_security.rs` | Who may open the pipe, and the client's check that it is the daemon's |
| `crates/mosaico-windows/src/daemon_ipc.rs` | Runs `serve()` on the daemon's IPC thread |
| `crates/mosaico-windows/src/ipc_api.rs` | `ApiPipeListener` -- the bar API pipe |
| `crates/mosaico-windows/src/ipc_api_io.rs` | `ApiPipeConnection`, `ApiPipeReader`, `ApiPipeWriter` -- overlapped reads and writes on one bar's connection |

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
{"version":2,"command":"Stop"}
```

```json
{"version":2,"command":"Action","action":"focus-left"}
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
| carried out, but the reply is larger than the limit | `too-large`, in place of the reply |

Bump `PROTOCOL_VERSION` whenever a change would make one side misread the
other. Version 2 added the `goto-workspace-next` and `goto-workspace-prev`
actions, which a daemon of version 1 cannot read.

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
  returns a `DaemonStatus`. `Inspect` returns a `DesktopSnapshot`: every
  monitor, workspace and managed window, the disconnected monitors and what
  has focus. Its `message` holds the same state as text, which `mosaico debug
  state` prints as the daemon wrote it; the CLI renders the payload itself
  only when a daemon sends no text.

`code` and `data` are optional, so replies from older daemons still parse.

A command the daemon understood but could not carry out is answered
`failed`, built with `Response::failed()` from the error, so `message` says
what went wrong. Today that is a `retile` whose monitors could not be read. A
command with nothing to act on, such as focusing left at the left edge, is not
a failure: it does nothing and answers `ok`. Window moves and focus changes go
through the applier, which may hold them back past the reply, so their
failures are logged rather than reported.

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
- only clients of the same user, on the same machine, may connect, and none
  less trusted than the daemon. If another program already holds the
  endpoint's name, creating it fails and `accept` reports the error, so the
  daemon never shares an endpoint it did not create

`IpcClient::connect` also refuses an endpoint that belongs to another user or
that a less trusted program created, with `IpcError::Untrusted`, before
anything is sent, and the endpoint cannot act as the client.

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
  deadline passes. A cancel only reaches I/O that has already started, so
  once the deadline has passed the watchdog repeats it every 10 ms until the
  call returns and the watchdog is disarmed. A reply bigger than the 4096
  byte pipe buffer blocks in `WriteFile` until the client reads; a smaller
  one blocks in `FlushFileBuffers`.
- `PipeConnection::finish` calls `FlushFileBuffers` before
  `DisconnectNamedPipe`. Disconnecting first throws away the unread reply and
  the client gets error 233.
- `PipeClient::connect` opens the pipe with `CreateFileW`. `ERROR_FILE_NOT_FOUND`
  means no daemon. On `ERROR_PIPE_BUSY` it waits up to 2 seconds with
  `WaitNamedPipeW` and tries once more. A pipe still busy after that is a
  daemon that is not answering, reported as `IpcError::TimedOut`. A pipe that
  goes away during the wait (`ERROR_FILE_NOT_FOUND`) is a daemon that exited,
  reported as `IpcError::NotRunning`.
- `PipeClient::is_daemon_running` uses `WaitNamedPipeW` with a 1 ms timeout.
  Only `ERROR_FILE_NOT_FOUND` means no daemon; a timeout means every instance
  is busy and the daemon is running.
- `PipeClient::connect` returns a `PipeStream`, whose reads use the same
  watchdog as the server to enforce the reply timeout.
- The pipe name comes from the instance (`pipe_name`): `\\.\pipe\mosaico` for
  the default instance, `\\.\pipe\mosaico-<name>` for a named one.

### Who may use the pipe

A pipe created without a descriptor lets Everyone and Anonymous read it.
`ipc_security.rs` gives every instance an explicit one instead:

```text
O:<user>D:P(D;;GA;;;NU)(A;;GA;;;SY)(A;;GA;;;<user>)S:(ML;;NWNR;;;<daemon's level>)
```

- The DACL allows the user and SYSTEM, denies network logons, and is
  protected from inheritance. Every instance also has
  `PIPE_REJECT_REMOTE_CLIENTS`.
- The mandatory label is the daemon's own integrity level, with no-read-up
  and no-write-up, so a process of the user at a lower level (a low-integrity
  or sandboxed program) can neither read nor write the pipe.
- The owner is the user, not the token's default owner, which is the
  Administrators group for an elevated process, so clients can check it either
  way.
- The first instance is created with `FILE_FLAG_FIRST_PIPE_INSTANCE`. If
  another program already holds the name, the daemon's IPC listener fails and
  the daemon shuts down instead of joining a pipe it does not control.

`PipeClient` opens the pipe with `SECURITY_SQOS_PRESENT |
SECURITY_IDENTIFICATION`, so the server can tell who the client is but cannot
impersonate it. It then reads the pipe's owner and label (`GetSecurityInfo`)
and refuses, with `IpcError::Untrusted`, a pipe owned by another account or
labelled below medium. A pipe with no label counts as medium, as the system
treats it. The owner is compared as a SID (`GetSecurityDescriptorOwner`),
never as descriptor text, which names some accounts by an alias: the built-in
Administrator is `LA` there. A label exactly at the daemon's level is not
required: an elevated CLI still reaches an unelevated daemon.

This keeps out other users, other machines and less trusted processes of the
user. A process of the same user at the same integrity can still create an
instance of the pipe once the daemon has made the first one; that is the
user's own code, as with any per-user IPC.

An elevated daemon labels its pipe high. An unelevated CLI then cannot open
it, as before this descriptor: the default one only let administrators and the
owner, the Administrators group, write. Run the CLI elevated as well to reach
an elevated daemon.

## The Bar API Pipe

The bar API has a pipe of its own, `api_pipe_name(instance)`:
`\\.\pipe\mosaico.bar`, or `\\.\pipe\mosaico-<name>.bar` for a named
instance. An instance name cannot hold a dot, so it never collides with an
instance called `bar`. Core defines what the platform provides
(`ApiListener`, `ApiConnection`, `ApiReader`, `ApiWriter` in
`crates/mosaico-core/src/api/connection.rs`); the sessions run in core.
`ApiPipeListener` differs from `PipeServer` where a session differs from a
request:

- Instances are opened with `FILE_FLAG_OVERLAPPED`. On a synchronous handle
  one call runs at a time, so a read waiting for the bar would hold up every
  write to it. Each call starts, then waits on its own event beside the
  connection's shutdown event and its deadline; a call given up on is
  cancelled with `CancelIoEx`, and its result is waited for before its
  buffer goes away.
- A write that has not finished by the session's write deadline fails with
  `TimedOut`, and the session is dropped. The output buffer is 256 KiB, a
  few full states, so a bar busy for a moment does not hold up its session;
  the input buffer is 16 KiB.
- `ConnectNamedPipe` is overlapped too, and waits beside a stop event, so
  the daemon stops the listener at once. A spare instance waits for the next
  bar while the others are served.
- Shutting a connection down wakes its reader and refuses further writes,
  and the instance is closed rather than disconnected, so the `bye` that was
  written stays readable and the bar then reads the end.
- Same descriptor as the CLI's pipe, with one difference: the mandatory
  label is always medium (`PipeSecurity::at_medium()`), whatever the
  daemon's level, since a bar is an ordinary program of the user, so an
  ordinary bar reaches an elevated daemon. Low-integrity programs, other
  users and other machines stay out, and the first instance is still created
  with `FILE_FLAG_FIRST_PIPE_INSTANCE`.

## Testing

Framing, request decoding, error replies and the accept loop are tested in
core against in-memory streams (`crates/mosaico-core/src/ipc/tests/`), so none
of it needs a daemon or a pipe.

`crates/mosaico-windows/src/ipc_tests.rs` runs a real pipe server on a
unique pipe name and checks that a daemon busy with one client still answers
the next. `ipc_security_tests.rs` checks, on pipes of its own, the owner and
label of the daemon's pipe, that the daemon will not join a pipe created
before it, and that a client refuses a low-integrity pipe. `ipc_api_tests.rs`
runs the bar API pipe for real: a whole session with `bye` readable after
the daemon closes, the listener stopping when told, the write deadline, and a
read returning when the other half shuts down.

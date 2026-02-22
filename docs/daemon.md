# Daemon & Event Loop

The daemon is the long-running background process that manages window tiling.
It orchestrates the Win32 event loop, hotkeys, IPC listener, and the tiling
manager, all on separate threads unified by a single `mpsc` channel.

## Architecture

The daemon uses a multi-thread architecture:

1. **Main thread** -- runs `TilingManager` logic, processes all messages
2. **Event loop thread** -- runs `SetWinEventHook` + `GetMessageW` pump,
   handles hotkey registration and dispatch
3. **IPC thread** -- runs blocking named pipe server, accepts CLI commands
4. **Config watcher thread** -- polls config files for changes every 2 seconds

Three additional bridge threads forward events, actions, and config reloads
from separate `mpsc` channels into the unified `DaemonMsg` channel.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-windows/src/daemon.rs` | `run()`, `daemon_loop()`, `ipc_loop()`, `DaemonMsg` |
| `crates/mosaico-windows/src/event_loop.rs` | `start()`, `EventLoopHandle`, `run_message_pump()`, `win_event_proc()` |
| `crates/mosaico-windows/src/config_watcher.rs` | `watch()`, `ConfigReload` -- polls config files for changes |

### Key Types

- `DaemonMsg` -- unified message enum with four variants:
  - `Event(WindowEvent)` -- window state change from Win32
  - `Action(Action)` -- user action from hotkey
  - `Command(Command, ResponseSender)` -- CLI command from IPC with reply channel
  - `Reload(ConfigReload)` -- validated config change from file watcher
- `EventLoopHandle` -- contains the thread ID and `JoinHandle`; `.stop()`
  posts `WM_QUIT` to terminate the message pump

## Startup Sequence

`daemon::run()` is the entry point, called from the hidden `daemon` CLI
subcommand:

1. Enable per-monitor DPI awareness
2. Write PID file to `%LOCALAPPDATA%\mosaico\mosaico.pid`
3. Call `daemon_loop()` for the main logic
4. On exit, remove PID file

`daemon_loop()` performs:

1. Load configuration (`config.toml`, `keybindings.toml`, `rules.toml`)
2. Initialize the logger
3. Create `TilingManager` with layout, rules, and border config
4. Create `mpsc` channels for events, actions, and the unified daemon channel
5. Start the event loop thread (registers `SetWinEventHook` + hotkeys)
6. Start bridge threads to forward events and actions into `DaemonMsg`
7. Start the IPC thread
8. Start the config file watcher thread
9. Start a reload bridge thread to forward `ConfigReload` into `DaemonMsg`
10. Enter the main receive loop

## Main Loop

The main thread runs a `recv_timeout(100ms)` loop that dispatches messages:

- `DaemonMsg::Event(event)` -- forwarded to `TilingManager::handle_event()`
- `DaemonMsg::Action(action)` -- forwarded to `TilingManager::handle_action()`
- `DaemonMsg::Command(cmd, reply)` -- handles `Stop` (breaks loop),
  `Status` (replies ok), `Action` (forwards to tiling manager)
- `DaemonMsg::Reload(ConfigReload::Config(cfg))` -- calls
  `TilingManager::reload_config()` to apply new layout/border settings
- `DaemonMsg::Reload(ConfigReload::Rules(rules))` -- calls
  `TilingManager::reload_rules()` to replace window rules

The 100ms timeout prevents busy-waiting while allowing responsive shutdown.

## Event Loop Thread

`event_loop::start()` spawns the Win32 message pump thread:

1. Registers `SetWinEventHook` with a callback (`win_event_proc`) for the
   event range `EVENT_MIN` to `EVENT_MAX`
2. Creates a `HotkeyManager` and registers all keybindings as global hotkeys
3. Runs `GetMessageW` loop:
   - `WM_HOTKEY` messages are dispatched to `HotkeyManager::dispatch()`
   - All other messages go through `TranslateMessage`/`DispatchMessageW`

The `win_event_proc` callback is an `extern "system"` function that uses
thread-local storage (`EVENT_SENDER`) to access the `mpsc::Sender<WindowEvent>`
since Win32 callbacks cannot capture Rust closures.

### Win32 Event Hook Configuration

- Range: `EVENT_MIN=0x0003` to `EVENT_MAX=0x800C`
- Flags: `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS`
- `OUTOFCONTEXT` means the callback runs in Mosaico's process
- `SKIPOWNPROCESS` prevents Mosaico's own windows (borders) from triggering
  events

## IPC Thread

`daemon::ipc_loop()` runs on a dedicated thread:

1. Creates a `PipeServer` (named pipe `\\.\pipe\mosaico`)
2. Blocks on `accept_command()`
3. Wraps the command as `DaemonMsg::Command(cmd, reply_tx)`
4. Sends to the main thread's unified channel
5. Waits for the reply on `reply_rx`
6. Sends the response back through the pipe

## Config File Watcher

The config watcher runs on a dedicated thread, polling `config.toml` and
`rules.toml` for modification time changes every 2 seconds.

### Key Types

- `ConfigReload` (enum) -- `Config(Config)` or `Rules(Vec<WindowRule>)`

### Behavior

1. Records the initial modification time of each file
2. On each poll cycle, compares current mtime with the stored value
3. If changed, validates the new content using `try_load()` or
   `try_load_rules()`
4. Only valid configs are sent as `ConfigReload` -- invalid files are logged
   and skipped
5. The reload is forwarded via a bridge thread to `DaemonMsg::Reload`

Keybindings are **not** hot-reloaded because `RegisterHotKey` requires
unregistering and re-registering on the event loop thread. A daemon restart
is required for keybinding changes.

## Shutdown

Shutdown is triggered by:

- A `Stop` command received via IPC
- The main loop breaks, which triggers cleanup:
  - `EventLoopHandle::stop()` posts `WM_QUIT` to the event loop thread
  - `HotkeyManager::Drop` unregisters all hotkeys
  - `SetWinEventHook` is unhooked
  - PID file is removed
  - Bridge threads (event, action, reload) are joined

## Design Decisions

- Three-thread architecture separates concerns: Win32 APIs require a message
  pump on the same thread, blocking pipe I/O needs its own thread, and tiling
  logic runs on the main thread.
- Bridge threads exist because Win32 events and hotkeys produce different
  channel types that need to be unified into `DaemonMsg`.
- `recv_timeout(100ms)` balances responsiveness with CPU efficiency.
- Thread-local `EVENT_SENDER` is the only way to pass data from a Win32
  callback to Rust code since the callback signature is fixed by the OS.

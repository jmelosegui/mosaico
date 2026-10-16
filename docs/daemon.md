# Daemon & Event Loop

The daemon is the long-running background process that manages window tiling.
It orchestrates window events, hotkeys, focus-follows-mouse, the IPC listener,
the status bar, and the tiling manager, all on separate threads unified by a
single `mpsc` channel.

## Architecture

The daemon uses a multi-thread architecture:

1. **Main thread** -- runs `TilingManager` + `BarManager` logic, processes
   all messages
2. **Window event thread** -- owns the `SetWinEventHook` hooks and the hidden
   window for display and work-area broadcasts, and runs the `GetMessageW`
   pump that delivers them (`WindowsEventSource`)
3. **Hotkey thread** -- registers the keybindings with `RegisterHotKey` and
   dispatches `WM_HOTKEY` (`WindowsHotkeySource`)
4. **Focus-follows-mouse thread** -- polls the pointer every 100 ms, and only
   exists while `focus_follows_mouse` is on (`FocusFollowsMouse`)
5. **IPC thread** -- runs blocking named pipe server, accepts CLI commands
6. **Config watcher thread** -- waits for changes in the config directory and
   reloads the config files that changed
7. **Tick thread** -- sends a `DaemonMsg::Tick` every 1 second for bar
   widget refresh (clock, CPU, RAM)

Three additional bridge threads forward events, actions, and config reloads
from separate `mpsc` channels into the unified `DaemonMsg` channel.

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/lifecycle.rs` | `run()`, `Startup` -- what the daemon does before and after its loop |
| `crates/mosaico-windows/src/daemon.rs` | `serve()`, `daemon_loop()`, `ipc_loop()`, `DaemonMsg` |
| `crates/mosaico-windows/src/daemon_inputs.rs` | `Inputs` -- starts and stops window events, hotkeys and focus-follows-mouse |
| `crates/mosaico-windows/src/event_source.rs` | `WindowsEventSource` -- the WinEvent hooks and the broadcast window |
| `crates/mosaico-windows/src/hotkey_thread.rs` | `WindowsHotkeySource` -- registers and dispatches hotkeys on a thread of its own |
| `crates/mosaico-windows/src/message_thread.rs` | `MessageThread` -- a thread with a message queue, stopped with `WM_QUIT` |
| `crates/mosaico-core/src/config/reload/` | `ConfigReloader`, `ConfigReload`, `ConfigWatch` -- which config files reload and how, and the watch loop |
| `crates/mosaico-windows/src/directory_watcher.rs` | `WindowsDirectoryWatcher` -- change notifications on the config directory |
| `crates/mosaico-windows/src/bar_manager.rs` | `BarManager` -- manages status bar windows across monitors |

### Key Types

- `DaemonMsg` -- unified message enum with five variants:
  - `Event(WindowEvent)` -- window state change from Win32
  - `Action(Action)` -- user action from hotkey
  - `Command(Command, ResponseSender)` -- CLI command from IPC with reply channel
  - `Reload(Box<ConfigReload>)` -- validated config change from file watcher
  - `Tick` -- 1-second timer for bar widget refresh
- `Inputs` -- holds the `EventWatch` of the event source, the `HotkeyWatch`
  of the hotkey source and `FocusFollowsMouse`, and starts the sources
  through the `EventSource` and `HotkeySource` traits; `.stop()` stops all
  three and waits for their threads

## Startup Sequence

The hidden `daemon` CLI subcommand runs `mosaico_core::lifecycle::run()` once
`main` has set the process up, as it does for every command
(`platform::init()`, which makes it per-monitor DPI aware). `platform.rs`
gives it a `Startup`: the instance, its single instance lock
(`SingleInstance`, a named mutex on Windows), the `Installer`, the running
executable, the process id, and the platform's loop, `daemon::serve()`. In
order, it:

1. Deletes what a self-update left beside the executable
   (`Installer::remove_leftovers()`), ignoring a failure
2. Takes the single instance lock, which is identified by the instance name;
   a second daemon of the same instance stops here with
   `Error::AlreadyRunning` ("Mosaico daemon is already running."), or with an
   `Error::Platform` when it may not open the other daemon's lock, such as
   another user's, leaving the running daemon's PID file alone
3. Writes the instance's PID file (`%LOCALAPPDATA%\mosaico\mosaico.pid` for
   the default instance)
4. Logs that it started
5. Calls `serve()`, which checks that the instance is the one the process
   resolved (`Instance::require_current`), since the loop still finds the
   logs and the record of hidden windows through `Instance::current()`. It
   then builds the instance's `ConfigStore` (`ConfigStore::for_instance`),
   through which the loop reads, watches and updates the config files, and
   runs `daemon_loop()` for the main logic
6. Logs the error the loop returned, if any, since the daemon has no
   console. Errors from the steps before it, and an instance that `serve()`
   refuses in step 5, come before the logger is set up and are only printed
   by the `daemon` subcommand
7. Removes the PID file, then releases the lock, so a daemon that starts next
   never has its PID file removed by this one. This also happens when a panic
   in the loop unwinds; release builds abort on a panic and leave the PID file
   behind, as a crash does

`daemon_loop()` performs:

1. Load configuration (`config.toml`, `keybindings.toml`, `rules.toml`) from
   the instance's `ConfigStore`
2. Initialize the logger, which then writes what was logged before it,
   such as problems found in the config (see [logging.md](logging.md))
3. Resolve the current theme from `config.theme`
4. Load `bar.toml` and create `BarManager` with monitor rects and theme
5. Create `TilingManager` with layout, rules, and border config
6. Adjust work areas for bar height on monitors that display a bar
7. Perform initial bar render with `bar_mgr.update(&manager.bar_states())`
8. Create `mpsc` channels for events, actions, and the unified daemon channel
9. Start the inputs: the window event source, the hotkey source and, when
   `focus_follows_mouse` is on, the focus-follows-mouse thread
10. Start bridge threads to forward events and actions into `DaemonMsg`
11. Start the IPC thread
12. Start the config file watcher thread
13. Start a reload bridge thread to forward `ConfigReload` into `DaemonMsg`
14. Start the tick thread (1-second interval for bar refresh)
15. Enter the main receive loop

## Main Loop

The main thread runs a blocking `recv()` loop that dispatches messages:

- `DaemonMsg::Event(event)` -- forwarded to `TilingManager::handle_event()`,
  then bar is updated
- `DaemonMsg::Action(action)` -- forwarded to `TilingManager::handle_action()`,
  then bar is updated
- `DaemonMsg::Command(cmd, reply)` -- handles `Stop` (breaks loop),
  `Status` (replies with window count), `Action` (forwards to tiling manager
  and updates bar)
- `DaemonMsg::Reload(ConfigReload::Config(cfg))` -- resolves the new theme,
  calls `TilingManager::reload_config()`, re-resolves bar colors, updates bar
- `DaemonMsg::Reload(ConfigReload::Rules(rules))` -- calls
  `TilingManager::reload_rules()` to replace window rules
- `DaemonMsg::Reload(ConfigReload::Bar(bar_cfg))` -- reloads the bar with
  `bar_mgr.reload()`, re-resolves colors, resets and adjusts work areas for
  the new bar height, then retiles and updates
- `DaemonMsg::Tick` -- calls `bar_mgr.update()` to refresh time-based
  widgets (clock, CPU, RAM)

The blocking `recv()` is CPU-efficient (no polling) and wakes immediately
when any message arrives.

## Input Threads

Win32 delivers WinEvent callbacks, `WM_HOTKEY` and window messages to the
thread that set each of them up, and that thread must pump messages. The
window events and the hotkeys therefore each run on a `MessageThread`, which
creates the thread's message queue before it reports ready, so nothing
posted to it is lost, and stops it by posting `WM_QUIT` and joining it.

### Window Event Thread

`WindowsEventSource::start()` (the core `EventSource` contract, see
[window-events.md](window-events.md)):

1. Installs two `SetWinEventHook` hooks with a callback (`win_event_proc`),
   for `0x0003` to `0x0017` and `0x8000` to `0x800C`
2. Creates the hidden window that receives `WM_DISPLAYCHANGE` and
   `WM_SETTINGCHANGE`
3. Runs a `GetMessageW` loop until it is stopped, then destroys the window and
   removes the hooks

If the hooks cannot be installed, `start` fails, and the daemon stops with
that error in its log. If only the hidden window cannot be created, the
source runs without display and work area changes, and the daemon logs
why.

The callbacks are `extern "system"` functions that use thread-local storage
(`EVENTS`) to reach the `mpsc::Sender<WindowEvent>`, since Win32 callbacks
cannot capture Rust closures.

### Win32 Event Hook Configuration

- Ranges: `0x0003` to `0x0017` and `0x8000` to `0x800C`
- Flags: `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS`
- `OUTOFCONTEXT` means the callback runs in Mosaico's process
- `SKIPOWNPROCESS` prevents Mosaico's own windows (borders) from triggering
  events

### Hotkey Thread

`WindowsHotkeySource` implements core's `HotkeySource`. Its `start()`
registers every keybinding through core's `Hotkeys` and `Win32Hotkeys`,
which calls `RegisterHotKey` with no window, so `WM_HOTKEY` lands in this
thread's queue, and runs a pump that dispatches `WM_HOTKEY` before anything
else. Bindings that cannot be registered are logged at warn. The
`HotkeyWatch` it returns posts pausing and unpausing to the thread as
`WM_HOTKEY_PAUSE` thread messages, and `Hotkeys` decides which hotkeys a
pause keeps. Nothing else is queued on this thread, so a flood of window
events cannot hold a shortcut back.

### Focus-Follows-Mouse Thread

`FocusFollowsMouse` (core) polls the window under the pointer every 100 ms
while `focus_follows_mouse` is on, and sends `MouseHover` once the pointer has
rested on a window for 3 polls. A config reload that turns the setting off
stops and joins the thread; one that turns it on starts a new one.

## IPC Thread

`daemon_ipc::ipc_loop()` runs `mosaico_core::ipc::serve()` on a dedicated
thread, with a `PipeServer` on `DEFAULT_PIPE_NAME` (`\\.\pipe\mosaico`):

1. Waits for a CLI connection and reads its request
2. Answers malformed, unknown or mismatched requests itself
3. Wraps any other command as `DaemonMsg::Command(cmd, reply_tx)` and sends it
   to the main thread
4. Waits for the reply on `reply_rx` and sends it back through the pipe

If the pipe can no longer accept connections, the thread sends
`DaemonMsg::IpcFailed` and the daemon shuts down cleanly. See [ipc.md](ipc.md).

## Config File Watcher

The config watcher runs on a dedicated thread. It sleeps until something in
the config directory may have changed, then reloads `config.toml`,
`user-rules.toml` and `bar.toml` if their modification time changed. The
files are not polled, and nothing waits for a burst of writes to settle.

### Key Types

- `ConfigReloader` (`mosaico_core::config`) -- decides which files reload and
  loads them
- `ConfigWatch` (`mosaico_core::config`) -- `open()` records the files and
  only then opens the platform watcher, so a change made while the watch
  starts is not lost; `run()` is the watch loop
- `ConfigReload` (enum) -- `Config(Box<Config>)`, `Rules(Vec<WindowRule>)`, or
  `Bar(Box<BarConfig>)`
- `DirectoryWatcher` (`mosaico_core::watch`) -- the platform contract: block
  until something in a directory may have changed, or until a timeout.
  `WindowsDirectoryWatcher` implements it with `FindFirstChangeNotificationW`
  (file names and last writes in the directory, not its subdirectories)

### Behavior

1. Records the modification time of each watched file, then starts watching
   the directory. A change made in between is reloaded with the first change
   the platform reports
2. Waits for a change for up to 5 seconds at a time, and checks between
   waits whether the daemon is stopping
3. When the platform reports a change, compares each file's modification time
   with the last one seen, in the order `config.toml`, `user-rules.toml`,
   `bar.toml`. A report with nothing changed does nothing
4. A changed `config.toml` or `bar.toml` is loaded with the daemon's
   `ConfigStore` (`try_load_config()` or `try_load_bar()`). Only a file that
   loads is sent as a `ConfigReload`; one that does not is logged and
   skipped, and is not read again until it changes again
5. A changed `user-rules.toml` reloads the merged rules with
   `load_merged_rules()`
6. The reload is forwarded via a bridge thread to `DaemonMsg::Reload`

`rules.toml` is not watched: the daemon downloads the community rules at
startup and sends their reload itself. Keybindings are **not** hot-reloaded
because `RegisterHotKey` requires unregistering and re-registering on the
hotkey thread. A daemon restart is required for keybinding changes.

If the config directory cannot be watched, for instance because it does not
exist, the watcher logs `FindFirstChangeNotificationW failed, watcher
exiting: <reason>` and the daemon runs without hot-reload. If the directory
is deleted while it is watched, the watcher logs the warning
`config watcher stopped: <reason>` and ends; restart the daemon once the
directory is back. If the directory is moved or renamed instead, the
watcher follows it to its new location while the daemon keeps reading the
old path. Nothing is logged when it moves, and hot-reload stops working
until the daemon restarts: a later change in the moved directory finds no
files at the old path, and reloads as if they had been deleted.

## Shutdown

Shutdown is triggered by:

- A `Stop` command received via IPC
- The main loop breaks, which triggers cleanup:
  - `TilingManager::restore_all_windows()` -- shows all hidden workspace
    windows so they are not left invisible after exit
  - `BarManager::hide_all()` -- hides all status bar overlay windows
  - `Inputs::stop()` posts `WM_QUIT` to the window event thread, which
    destroys its hidden window and removes its hooks, then to the hotkey
    thread, where dropping `Hotkeys` unregisters the hotkeys still
    registered, and stops the focus-follows-mouse thread; it waits for each
    of them
  - Once the loop has returned, `lifecycle::run` removes the PID file and then
    releases the single instance lock
  - Bridge threads (event, action, reload) and tick thread are joined
  - The config watcher thread is joined; while the directory is quiet it
    checks for the stop every 5 seconds, so this can take that long

### Unclean exit

A daemon that is killed or crashes never reaches `restore_all_windows()`, so
the windows it hid with a workspace switch stay hidden, and the next daemon
would skip them as cloaked or invisible. To recover them, the applier keeps a
record of every window hidden by switching (`mosaico_core::tiler::HiddenWindows`)
in `hidden-windows.json` in the instance's data directory, updated as hide and
show effects are applied and deleted once nothing is hidden. On startup,
before enumerating windows, `TilingManager::new` shows each recorded window
the way it was hidden, then clears the record. Window handles can be reused,
so an entry is only restored if the window still has the recorded process id
and class.

## Design Decisions

- Multi-thread architecture separates concerns: Win32 APIs require a message
  pump on the same thread, blocking pipe I/O needs its own thread, bar and
  tiling logic run on the main thread.
- Bridge threads exist because Win32 events and hotkeys produce different
  channel types that need to be unified into `DaemonMsg`.
- Blocking `recv()` is more CPU-efficient than `recv_timeout()` since the
  1-second tick thread provides the periodic wake-up needed for bar refresh.
- The `BarManager` is updated after every event, action, and command so that
  workspace indicators and window counts stay current.
- Theme tracking (`current_theme`) lives in the daemon loop so that both
  config reloads and bar reloads can re-resolve named colors.
- Thread-local `EVENTS` is the only way to pass data from a Win32
  callback to Rust code since the callback signature is fixed by the OS.
- Window events, hotkeys and focus-follows-mouse each have a thread of their
  own, so each is a separate contract a platform can implement on its own,
  and none of them waits behind another's messages.

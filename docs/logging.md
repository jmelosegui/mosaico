# Logging System

Mosaico includes a custom file-based logger with size-based rotation. It has
no external logging crate dependencies, keeping the dependency tree minimal.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/log.rs` | `LogConfig`, `Level`, `init()`, `write()`, macros |
| `crates/mosaico-core/src/log_file.rs` | `Logger`: the log file, its line format and rotation |
| `crates/mosaico-core/src/log_backlog.rs` | `Backlog`: lines logged before `init()` |

### Key Types

- `LogConfig` -- configuration: `enabled: bool`, `level: String`,
  `max_file_mb: u64`
- `Level` (enum) -- `Debug`, `Info`, `Warn`, `Error` (implements `Ord` for
  filtering)
- `Logger` (struct) -- holds the file handle, path, minimum level, max size,
  and bytes written
- `Backlog` (struct) -- up to 100 lines, with their level, logged before
  `init()`

## Configuration

In `config.toml`:

```toml
[logging]
enabled = false    # Enable file logging (disabled by default)
level = "info"     # Minimum log level: debug, info, warn, error
max_file_mb = 10   # Maximum log file size in MB before rotation
```

Logging is disabled by default to avoid disk writes for casual users.

## Log File Location

Logs are written to `logs/mosaico.log` in the instance's config directory:
`~/.config/mosaico/logs/mosaico.log` for the default instance. The `logs/`
directory is created automatically if it does not exist. Without a resolved
instance there is no log file, so nothing is logged: `init()` leaves logging
off and drops the lines kept so far. The CLI resolves the instance at startup,
before the daemon reads its config.

## Initialization

`init(config)`:

1. Returns at once if the logger was already set up: only the first call counts
2. Parses the configured level string
3. If logging is enabled and the instance of the process has been resolved,
   creates the log directory and opens or creates the log file
4. Stores the `Logger` in a global `OnceLock<Option<Mutex<Logger>>>`, `None`
   when logging is disabled, the instance is not resolved or the file cannot
   be opened
5. Writes an initialization log message with the configured level and max size
6. Writes the backlog: the lines logged before `init()`, at or above the
   configured level, in the order they were logged

Initialization is called once in `daemon_loop()` after loading configuration.
The `OnceLock` ensures thread-safe single initialization.

`init()` writes to the instance's log directory. `init_in(config, dir)` does the
same with a directory of its own, which is how the tests in
`crates/mosaico-core/tests/` (`log_early_lines.rs`, `log_disabled.rs`) keep
their log away from the user's. Each of them runs in a process of its own, as
the logger can only be set up once.

## Writing Logs

`write(level, args)`:

1. Before `init()`: keeps the line in the backlog (see below) and returns
2. If logging is disabled: returns
3. Acquires the mutex lock
4. Checks if `level >= min_level`; skips if below threshold
5. Formats the log line: `[TIMESTAMP] LEVEL message\n`
6. Writes to the file and flushes
7. Tracks bytes written; triggers rotation if `max_bytes` is exceeded

### Backlog

The daemon has to read `config.toml` before it knows whether to log, so
anything it logs while starting up comes before `init()`. Those lines are kept
in a backlog of at most 100 lines, already formatted with the time they were
logged, and `init()` writes them after its own line. When logging is disabled
they are dropped. A process that never calls `init()`, such as the CLI, keeps
at most those 100 lines and does not even format the ones after them.

### Timestamp

`timestamp()` produces a simple UTC `HH:MM:SS` timestamp using
`std::time::SystemTime`. No external chrono dependency is used.

## Log Rotation

When the log file exceeds the configured maximum size:

1. `rotate()` closes the current file
2. Renames `mosaico.log` to `mosaico.log.1` (overwriting any existing backup)
3. Opens a fresh `mosaico.log`
4. Resets the `written` counter

Only one backup file is kept to limit disk usage.

## Macros

Four convenience macros are provided:

- `log_debug!(format, args...)` -- logs at `Debug` level
- `log_info!(format, args...)` -- logs at `Info` level
- `log_warn!(format, args...)` -- logs at `Warn` level
- `log_error!(format, args...)` -- logs at `Error` level

These are used throughout `tiling.rs` for window add/remove/event logging
and in `mosaico_core::lifecycle` for startup/shutdown information.

## Where Errors Go

The daemon runs without a console, so everything it reports goes through
these macros: hotkeys that cannot be registered, a missing event sink window,
a bar that cannot be created, and an error from the daemon loop, which
`lifecycle::run()` logs before returning it. Errors before the loop, such as
another daemon holding the single instance lock or an instance that
`serve()` refuses, happen before the logger is set up, so they only reach
`mosaico daemon`'s standard error.

The CLI and the daemon load the config the same way, so the problems found
while loading it go to both places: to standard error, where commands such as
`mosaico doctor` and `mosaico debug list` have always shown them as
`Warning: ...` and `Info: ...` lines, and to the log. That covers a file that
cannot be read or parsed and falls back to defaults, a `[workspaces.layouts]`
key that is not a workspace number, and a migrated legacy section. The
`merge_missing_*` helpers, which only the daemon calls, log alone. CLI
commands keep printing their own messages to the terminal.

## Design Decisions

- **Custom implementation** over external crates (`log`, `tracing`, etc.)
  keeps the dependency count minimal for a system-level tool.
- **Global `OnceLock<Option<Mutex<Logger>>>`** provides thread-safe access
  from any thread (window events, hotkeys, IPC, main) without passing
  references.
- **Disabled by default** avoids unexpected disk writes for users who just
  want a tiling manager.
- **Single backup rotation** is simple and sufficient for diagnostics without
  consuming significant disk space.
- **UTC timestamps** avoid timezone complexity. `SystemTime` is available in
  the standard library with no extra dependencies.
- **Flush after every write** ensures log data is persisted immediately, which
  is important for debugging crashes.

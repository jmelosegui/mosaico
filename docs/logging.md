# Logging System

Mosaico includes a custom file-based logger with size-based rotation. It has
no external logging crate dependencies, keeping the dependency tree minimal.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico-core/src/log.rs` | `LogConfig`, `Level`, `Logger`, `init()`, `write()`, macros |

### Key Types

- `LogConfig` -- configuration: `enabled: bool`, `level: String`,
  `max_file_mb: u64`
- `Level` (enum) -- `Debug`, `Info`, `Warn`, `Error` (implements `Ord` for
  filtering)
- `Logger` (struct) -- holds the file handle, path, minimum level, max size,
  and bytes written

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

Logs are written to `~/.config/mosaico/logs/mosaico.log`. The `logs/`
directory is created automatically if it does not exist.

## Initialization

`init(config)`:

1. Checks if logging is enabled; returns early if not
2. Creates the log directory
3. Opens or creates the log file
4. Parses the configured level string
5. Stores the `Logger` in a global `OnceLock<Mutex<Logger>>`
6. Writes an initialization log message with the configured level and max size

Initialization is called once in `daemon_loop()` after loading configuration.
The `OnceLock` ensures thread-safe single initialization.

## Writing Logs

`write(level, args)`:

1. Checks if the logger is initialized (no-op if not)
2. Acquires the mutex lock
3. Checks if `level >= min_level`; skips if below threshold
4. Formats the log line: `[TIMESTAMP] LEVEL message\n`
5. Writes to the file and flushes
6. Tracks bytes written; triggers rotation if `max_bytes` is exceeded

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
and in `daemon.rs` for startup/shutdown information.

## Design Decisions

- **Custom implementation** over external crates (`log`, `tracing`, etc.)
  keeps the dependency count minimal for a system-level tool.
- **Global `OnceLock<Mutex<Logger>>`** provides thread-safe access from any
  thread (event loop, IPC, main) without passing references.
- **Disabled by default** avoids unexpected disk writes for users who just
  want a tiling manager.
- **Single backup rotation** is simple and sufficient for diagnostics without
  consuming significant disk space.
- **UTC timestamps** avoid timezone complexity. `SystemTime` is available in
  the standard library with no extra dependencies.
- **Flush after every write** ensures log data is persisted immediately, which
  is important for debugging crashes.

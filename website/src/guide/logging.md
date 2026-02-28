# Logging

Mosaico includes a built-in file logger with automatic rotation. Logging is
disabled by default.

## Configuration

Enable logging in `config.toml`:

```toml
[logging]
enabled = true
level = "info"        # debug, info, warn, error
max_file_mb = 10      # Max file size before rotation
```

## Log Levels

| Level | Description |
|-------|-------------|
| `debug` | Verbose output for troubleshooting |
| `info` | General operational messages |
| `warn` | Potential issues |
| `error` | Failures that affect functionality |

## Log Location

Logs are written to:

```
~/.config/mosaico/logs/mosaico.log
```

## Rotation

When the log file exceeds `max_file_mb`, it is rotated:

- The current file is renamed to `mosaico.log.1`
- A new `mosaico.log` is created
- Only one backup file is kept

## When to Enable Logging

Logging is most useful when:

- Debugging why a window is not being tiled
- Investigating unexpected behavior
- Reporting issues on GitHub

For general use, logging can remain disabled to avoid unnecessary disk
writes.

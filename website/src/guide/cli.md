# CLI Commands

All interactions with Mosaico go through the command-line interface.

## `mosaico init`

Creates default configuration files in `~/.config/mosaico/`:

- `config.toml` -- layout, border, and logging settings
- `keybindings.toml` -- keyboard shortcuts
- `rules.toml` -- window management rules

Files that already exist are skipped to preserve your customizations.

## `mosaico start`

Launches the daemon as a detached background process. On success, a startup
banner is printed showing the config directory, PID, and a usage tip. If a
newer release is available on GitHub, an update notice is shown.

If the daemon is already running, the command reports the existing state
instead of starting a second instance.

## `mosaico stop`

Sends a stop command to the running daemon. All windows are restored to
their original positions before the daemon exits. If the daemon is
unresponsive, a fallback mechanism terminates the process directly.

## `mosaico status`

Reports whether the daemon is running, along with its PID. Detects and
cleans up stale PID files from crashed sessions.

## `mosaico doctor`

Runs a comprehensive health check and prints colored status tags for each
item:

| Tag | Meaning |
|-----|---------|
| `[ok]` | Check passed |
| `[warn]` | Non-critical issue |
| `[fail]` | Critical problem |
| `[fixed]` | Auto-remediated |

Checks performed:

1. Config directory exists (creates it if missing)
2. `config.toml` syntax validation
3. `keybindings.toml` syntax validation
4. Key names resolve to valid key codes
5. `rules.toml` syntax validation
6. `bar.toml` syntax validation
7. Daemon status (IPC pipe, PID file, process liveness)
8. Monitor enumeration and dimensions

## `mosaico action <verb> [direction]`

Sends a tiling action to the running daemon. Available actions:

```sh
mosaico action focus left|right|up|down
mosaico action move left|right|up|down
mosaico action retile
mosaico action toggle-monocle
mosaico action close-focused
mosaico action goto-workspace <1-8>
mosaico action send-to-workspace <1-8>
```

These are the same actions triggered by keyboard shortcuts, useful for
scripting or integration with other tools.

## `mosaico debug list`

Displays a formatted table of all visible windows showing:

- HWND (hex)
- Managed status
- Monitor number
- Title and class name
- Position and dimensions
- Border offset

## `mosaico debug events`

Streams real-time window events to stdout. Press `Ctrl+C` to stop. Useful
for understanding how Mosaico reacts to window changes.

## `mosaico debug move <hwnd> <x> <y> <width> <height>`

Moves a specific window to exact pixel coordinates. The HWND can be provided
in decimal or `0x` hex format. Useful for testing window positioning.

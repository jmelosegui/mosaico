# Debug Tools

Mosaico provides debugging subcommands for inspecting and interacting with
the window manager's view of the desktop, plus a `doctor` command for health
checks. These run independently of the daemon and are useful for
troubleshooting.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico/src/commands/debug/list.rs` | `mosaico debug list` handler |
| `crates/mosaico/src/commands/debug/events.rs` | `mosaico debug events` handler |
| `crates/mosaico/src/commands/debug/move_window.rs` | `mosaico debug move` handler |
| `crates/mosaico/src/commands/doctor.rs` | `mosaico doctor` handler |

## `mosaico debug list`

Displays a formatted table of all visible windows on the desktop.

### Output Columns

| Column | Description |
|--------|-------------|
| HWND | Window handle in hex format |
| Managed | Whether the window would be tiled (`yes`/`no`) |
| Monitor | Monitor number (1-based) |
| Title | Window title text |
| Class | Window class name |
| Path | Full path of the owning executable |
| X, Y | Window position |
| Width, Height | Window dimensions |
| Border Offset | Invisible frame border offsets |

### Behavior

1. Enables DPI awareness for accurate coordinates
2. Loads window rules from configuration
3. Enumerates all visible windows via `EnumWindows`
4. For each window: queries title, class, executable path, rect, border
   offset, monitor, and evaluates rules. Windows without a caption are
   listed too, and show as managed only when a rule opts them in
5. Formats the results in a table using `comfy-table`
6. Prints a footer: `"Found N windows"` and monitor count

This command is useful for:
- Verifying which windows Mosaico would manage
- Checking window class names for writing rules
- Confirming DWM border offsets
- Finding window ids for `debug move`

## `mosaico debug events`

Streams real-time window events to stdout.

### Behavior

1. Enables DPI awareness
2. Creates an event channel
3. Starts the Win32 event loop (without hotkey registration). If the hidden
   window that receives display and work area changes cannot be created, it
   prints a warning saying why and goes on without those events
4. Registers a Ctrl+C handler that gracefully stops the event loop
5. Prints each received event: event name, HWND (hex), and window title

### Output Format

```
Created  0x001A0B2C  "Untitled - Notepad"
Focused  0x001A0B2C  "Untitled - Notepad"
Moved    0x001A0B2C  "Untitled - Notepad"
```

This command is useful for:
- Understanding which Win32 events fire for specific window operations
- Debugging why a window is or isn't being managed
- Verifying event translation from raw Win32 codes to `WindowEvent` variants

### Graceful Shutdown

The `ctrlc` crate is used to register a signal handler. On Ctrl+C, the handler
calls `EventLoopHandle::stop()`, which posts `WM_QUIT` to the event loop
thread, causing a clean exit.

## `mosaico debug move --window <id> --x <x> --y <y> --w <width> --h <height>`

Moves a specific window to exact pixel coordinates.

### Arguments

- `--window` -- Window id as `debug list` shows it (`0x` hex), or the same
  number in decimal. `--hwnd` is still accepted for it
- `--x`, `--y` -- Target position of the window's visible frame
- `--w`, `--h` -- Target width and height

### Behavior

1. Enables DPI awareness
2. Reads the id with `WindowId::parse`. Anything else is refused with exit
   status 1
3. Calls `WindowControl::place` with the rectangle (on Windows this includes
   invisible frame compensation)
4. Prints `Done.`, or the error with exit status 1 when the window cannot be
   moved

This command is useful for:
- Testing window positioning with frame compensation
- Verifying that a placement produces the expected visual result
- Manually arranging a specific window for debugging

## `mosaico doctor`

Runs a comprehensive health check of the Mosaico installation. Prints
colored status tags:

| Tag | Color | Meaning |
|-----|-------|---------|
| `[ok]` | Green | Check passed |
| `[warn]` | Yellow | Non-critical issue |
| `[fail]` | Red | Critical problem |
| `[fixed]` | Cyan | Auto-remediated |

### Checks Performed

1. **Config directory** -- verifies `~/.config/mosaico/` exists; creates it
   if missing (`[fixed]`)
2. **config.toml** -- validates TOML syntax via `try_load()`
3. **keybindings.toml** -- validates syntax via `try_load_keybindings()`
4. **Keybinding key codes** -- loads all keybindings and resolves each key
   name through `vk_from_name()`; reports the total count of valid bindings
   or lists any unresolvable key names
5. **rules.toml** -- validates syntax via `try_load_rules()`
6. **bar.toml** -- validates syntax via `try_load_bar()`
7. **Daemon status** -- checks IPC pipe connectivity, PID file, and process
   liveness; auto-removes stale PID files (`[fixed]`)
8. **Monitors** -- enumerates all connected monitors via
   `enumerate_monitors()` and prints per-monitor dimensions and positions

## Design Decisions

- All debug commands enable DPI awareness independently since they run
  outside the daemon process. Without this, coordinates would be virtualized
  on high-DPI displays.
- `debug list` evaluates the same rules as the daemon to show accurate
  "Managed" status.
- `debug events` runs without hotkeys to avoid interfering with the running
  daemon's hotkey registrations.
- `debug move` places the window through `WindowControl::place`, the same
  call the tiler's placements end in, so it accurately represents how the
  daemon positions windows. Unlike the tiler it does not skip a window that
  is already in place, and does not repaint the window afterwards.

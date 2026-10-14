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
| `crates/mosaico-core/src/tiler/enumerator.rs` | `Enumerator`, which `debug list` and `doctor` list windows and monitors through |

## `mosaico debug list`

Lists the windows on the desktop, one block per window, then the monitors.

### Output Columns

| Column | Description |
|--------|-------------|
| Id | Window id in hex format (the `HWND` on Windows) |
| Managed | Whether the window would be tiled (`yes`/`no`) |
| Monitor | Monitor number (1-based), or `?` when the window is on no listed monitor |
| State | `normal`, `maximized` or `minimized` |
| Title | Window title text |
| Class | Window class name |
| Path | Full path of the owning executable |
| X, Y | Window position |
| Width, Height | Window dimensions |
| Border Offset | Invisible frame border offsets |

### Behavior

1. Loads window rules from configuration
2. Lists the windows through the context's `Enumerator`
   (`mosaico_core::tiler::Enumerator`): the top-level application windows
   that are shown and not minimized, front to back. On Windows,
   `WindowsEnumerator` walks them with `EnumWindows`
3. For each window: asks `Enumerator::details` for its title, class,
   executable path, title bar, rect, state, monitor and border offset, and
   evaluates rules. Windows without a caption are listed too, and show as
   managed only when a rule opts them in
4. Prints a block per window, then `"N windows found"` and the monitors left
   to right (`Enumerator::monitors`) with their keys
5. If the windows cannot be listed, prints the error and exits with status 1

This command is useful for:
- Verifying which windows Mosaico would manage
- Checking window class names for writing rules
- Confirming DWM border offsets
- Finding window ids for `debug move`

## `mosaico debug events`

Streams real-time window events to stdout.

### Behavior

1. Creates an event channel
2. Starts the platform's window event source, which `platform.rs` picks
   (no hotkeys and no focus-follows-mouse). If the hidden window that
   receives display and work area changes cannot be created, it prints a
   warning saying why and goes on without those events
3. Asks to hear about Ctrl+C (`Interrupts`), so it can stop the event source
   gracefully; if that fails it prints a warning, and Ctrl+C ends it at once
4. Prints each received event: event name, HWND (hex), and window title,
   which it reads with `Enumerator::details` from the `Enumerator` that
   `platform::desktop()` picks

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

The command takes the platform's `Interrupts` from `platform::interrupts()`:
on Windows `ConsoleInterrupts`, a console control handler installed with
`SetConsoleCtrlHandler` in `ctrl_c.rs`, which reports Ctrl+C on a channel
instead of ending the process. The command then stops its `EventWatch`,
which posts `WM_QUIT` to the window event thread. The thread removes its
hooks and hidden window before the command exits.

## `mosaico debug move --window <id> --x <x> --y <y> --w <width> --h <height>`

Moves a specific window to exact pixel coordinates.

### Arguments

- `--window` -- Window id as `debug list` shows it (`0x` hex), or the same
  number in decimal. `--hwnd` is still accepted for it
- `--x`, `--y` -- Target position of the window's visible frame
- `--w`, `--h` -- Target width and height

### Behavior

1. Reads the id with `WindowId::parse`. Anything else is refused with exit
   status 1
2. Calls `WindowControl::place` with the rectangle (on Windows this includes
   invisible frame compensation)
3. Prints `Done.`, or the error with exit status 1 when the window cannot be
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
4. **Keybinding key codes** -- loads all keybindings and checks each key
   name with `Keybinding::combo()`, the check the daemon uses; reports the
   total count of valid bindings or lists any unknown key names
5. **rules.toml** -- validates syntax via `try_load_rules()`
6. **bar.toml** -- validates syntax via `try_load_bar()`
7. **Autostart** -- checks that what starts at login is the running mosaico
   executable with `start`: `[warn]` when nothing or another command is
   registered, or the entry is turned off in the startup apps settings,
   `[fail]` when the registered program is not a file, the entry is empty
   or the registration cannot be read. It never re-registers;
   `mosaico autostart enable` does
8. **Daemon status** -- checks IPC pipe connectivity, PID file, and process
   liveness; auto-removes stale PID files (`[fixed]`)
9. **Monitors** -- lists all connected monitors left to right via the
   context's `Enumerator::monitors()` and prints per-monitor dimensions and
   positions, in the physical pixels the daemon and `debug list` use

## Design Decisions

- Every mosaico process, so every debug command and `doctor` too, is made
  DPI aware once at startup by the platform's setup (`platform::init()` in
  `main`), since they run outside the daemon process. Without this,
  coordinates would be virtualized on high-DPI displays and would not match
  the daemon's.
- `debug list` evaluates the same rules as the daemon to show accurate
  "Managed" status.
- `debug events` runs without hotkeys to avoid interfering with the running
  daemon's hotkey registrations.
- `debug move` places the window through `WindowControl::place`, the same
  call the tiler's placements end in, so it accurately represents how the
  daemon positions windows. Unlike the tiler it does not skip a window that
  is already in place, and does not repaint the window afterwards.

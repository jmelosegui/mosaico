# Debug Tools

Mosaico provides three debugging subcommands for inspecting and interacting
with the window manager's view of the desktop. These run independently of the
daemon and are useful for troubleshooting.

## Architecture

### Key Files

| File | Purpose |
|------|---------|
| `crates/mosaico/src/commands/debug/list.rs` | `mosaico debug list` handler |
| `crates/mosaico/src/commands/debug/events.rs` | `mosaico debug events` handler |
| `crates/mosaico/src/commands/debug/move_window.rs` | `mosaico debug move` handler |

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
| X, Y | Window position |
| Width, Height | Window dimensions |
| Border Offset | Invisible frame border offsets |

### Behavior

1. Enables DPI awareness for accurate coordinates
2. Loads window rules from configuration
3. Enumerates all visible windows via `EnumWindows`
4. For each window: queries title, class, rect, border offset, monitor,
   and evaluates rules
5. Formats the results in a table using `comfy-table`
6. Prints a footer: `"Found N windows"` and monitor count

This command is useful for:
- Verifying which windows Mosaico would manage
- Checking window class names for writing rules
- Confirming DWM border offsets
- Identifying HWND values for `debug move`

## `mosaico debug events`

Streams real-time window events to stdout.

### Behavior

1. Enables DPI awareness
2. Creates an event channel
3. Starts the Win32 event loop (without hotkey registration)
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

## `mosaico debug move <hwnd> <x> <y> <width> <height>`

Moves a specific window to exact pixel coordinates.

### Arguments

- `hwnd` -- Window handle in decimal or `0x` hex format
- `x`, `y` -- Target position
- `width`, `height` -- Target dimensions

### Behavior

1. Enables DPI awareness
2. Parses the HWND (supports `0x` prefix for hex)
3. Creates a `Window` from the raw handle
4. Calls `set_rect()` with the specified rectangle (includes invisible frame
   compensation)
5. Reports success with the window title

This command is useful for:
- Testing window positioning with frame compensation
- Verifying that `set_rect()` produces the expected visual result
- Manually arranging a specific window for debugging

## Design Decisions

- All debug commands enable DPI awareness independently since they run
  outside the daemon process. Without this, coordinates would be virtualized
  on high-DPI displays.
- `debug list` evaluates the same rules as the daemon to show accurate
  "Managed" status.
- `debug events` runs without hotkeys to avoid interfering with the running
  daemon's hotkey registrations.
- `debug move` uses the same `Window::set_rect()` code path as the tiling
  manager, so it accurately represents how the daemon positions windows.

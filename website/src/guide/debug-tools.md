# Debugging

Mosaico provides several tools for inspecting and troubleshooting the
window manager.

## `mosaico doctor`

The `doctor` command performs a comprehensive health check:

```sh
mosaico doctor
```

It checks:

1. Config directory exists
2. All configuration files are valid TOML
3. All keybinding key names resolve to valid key codes
4. Daemon status (running, stopped, or stale)
5. Monitor enumeration and dimensions

Issues are reported with colored tags: `[ok]`, `[warn]`, `[fail]`, or
`[fixed]`.

## `mosaico debug list`

Lists all visible windows on the desktop:

```sh
mosaico debug list
```

The output is a formatted table showing:

| Column | Description |
|--------|-------------|
| HWND | Window handle in hex |
| Managed | Whether the window is tiled |
| Monitor | Which monitor the window is on |
| Title | Window title |
| Class | Window class name |
| Position | X, Y coordinates |
| Size | Width and height |
| Border Offset | Invisible frame compensation values |

This is useful for finding window class names when writing
[window rules](window-rules.md).

## `mosaico debug events`

Streams window events in real time:

```sh
mosaico debug events
```

Shows events like window creation, destruction, focus changes, moves,
minimize/restore, and title changes. Press `Ctrl+C` to stop.

This is useful for understanding how Mosaico reacts to window changes
and diagnosing timing issues.

## `mosaico debug move`

Moves a specific window to exact coordinates:

```sh
mosaico debug move 0x1A2B3C 100 200 800 600
```

Arguments: `<hwnd> <x> <y> <width> <height>`. The HWND can be in decimal
or `0x` hex format (use `debug list` to find it).

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
2. All configuration files are valid TOML, and how many click bindings
   `bar.toml` gives the bar, with any it ignores
3. Each value in `config.toml` and `bar.toml` that mosaico does not take as
   written (out of range, not one of the names listed, or an unknown key),
   named with the label the settings window gives it
4. All keybinding key names resolve to valid key codes
5. Daemon status (running, stopped, or stale)
6. Monitor enumeration and dimensions
7. Window transitions: the `[animations]` settings, whether Windows allows
   animations right now, and whether windows can move frame by frame

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
| Path | Path of the program that owns the window |
| Position | X, Y coordinates |
| Size | Width and height |
| Border Offset | Invisible frame compensation values |

This is useful for finding window class names and executable paths when
writing [window rules](window-rules.md).

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
mosaico debug move --window 0x1A2B3C --x 100 --y 200 --w 800 --h 600
```

`--window` is the id `debug list` shows (`0x` hex), or the same number in
decimal; `--hwnd` still works in its place. `--x` and `--y` are where the
window's visible frame goes, `--w` and `--h` its width and height. The
command exits with status 1 when the id is not a window id or the window
cannot be moved.

## `mosaico debug state`

Shows what the running daemon manages:

```sh
mosaico debug state
```

It prints each monitor and its workspaces that have windows, with every
window's id, whether it is visible, minimized, cloaked or elevated, its
position and size, class and title. Then it lists the disconnected
monitors, whether Mosaico runs elevated, and which monitor and window have
focus. The command exits with status 1 when Mosaico is not running.

This is useful for checking which windows Mosaico tiles, on which monitor
and workspace, and which window it considers focused.

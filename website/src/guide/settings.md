# Settings Window

The settings window lets you change mosaico without editing TOML files. It is a
separate program, `mosaico-settings.exe`, installed beside `mosaico.exe`.

## Opening It

- Click the mosaico icon in the notification area, or pick **Settings…** in
  its menu.
- Run `mosaico settings` from a terminal.
- Double-click `mosaico-settings.exe` in the install folder.

The daemon does not need to be running, and starting the daemon does not open
the window. Only one settings window opens at a time: opening it again brings
the one already open to the front.

The window tiles like any other window. To float it instead, add a rule to
`user-rules.toml`:

```toml
[[rule]]
match_path = "mosaico-settings.exe"
action = "float"
```

## Layout

The sidebar lists the pages and has a search field that filters the settings
on the current page by name and description. Under the pages it shows whether
the daemon is running and which version it is.

Each setting shows its default value, and a reset button appears next to any
setting that is not at its default. The footer counts your unsaved changes,
and **Revert** drops them all. Closing the window with unsaved changes, by its
close button, Alt+F4 or the taskbar, asks first: **Keep editing** goes back
to the window and **Discard and close** closes it.

## Saving

**Apply** writes your changes to the configuration file, and the running
daemon picks them up at once, as it does when you edit the file by hand: a
new theme recolors the borders and the bar without a restart. Only the
settings you changed are touched; comments, blank lines and the order of the
file are kept. A setting set back to its default is removed from the file, so
it follows the default if a later version changes it.

While you have no unsaved changes, the window follows edits made to the file
in an editor. If the file has a problem that would stop mosaico loading it,
the window says what it is and shows the defaults, and **Apply** stays
disabled until the file is fixed, so a broken file is never saved over.

## Tray Icon

While the daemon runs, the mosaico icon sits in the notification area.
Windows may put it in the overflow at first; drag it onto the taskbar to keep
it in view. Clicking it opens the settings window. Its menu has:

| Item | What it does |
|------|--------------|
| **Settings…** | Opens the settings window, as a click does |
| **Pause hotkeys** / **Resume hotkeys** | Same as `mosaico pause` and `mosaico unpause` |
| **Reload config** | Loads `config.toml`, `user-rules.toml` and `bar.toml` again |
| **Open config folder** | Opens the folder with the configuration files |
| **Open log** | Opens the log file, when logging is on |
| **Quit mosaico** | Stops the daemon, as `mosaico stop` does |

While the hotkeys are paused the icon is grayed with a red pause badge, and
its tooltip says so. When a configuration file does not load, the icon shows
a notification with the reason, and mosaico keeps the settings it has. The
icon comes back by itself when Explorer restarts. Set `enabled = false` under
`[tray]` in `config.toml` to hide it.

## General

| Section | Setting | File |
|---------|---------|------|
| Appearance | Theme and flavor | `config.toml` |
| Startup | Start at sign-in | Registry |
| Logging | Write a log file, level, maximum file size | `config.toml` |

Picking a theme or flavor previews it in the settings window right away.

## Current Limits

This release has the General page only; the other pages appear in the sidebar
but are not ready yet. Edit the [configuration files](configuration.md) to
change those settings for now.

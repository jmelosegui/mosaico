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

## Layout

| Section | Setting | Key in `config.toml` |
|---------|---------|----------------------|
| Tiling | Default layout, gap, split ratio | `layout.default`, `layout.gap`, `layout.ratio` |
| Other workspaces | Hide windows by | `layout.hiding` |

## Workspaces

| Section | Setting | Key in `config.toml` |
|---------|---------|----------------------|
| Switching | Whether a switch changes the focused monitor or every monitor, and wrapping around | `workspaces.mode`, `workspaces.wrap` |
| Layouts | The layout of each of the eight workspaces | `[workspaces.layouts]`, as `1 = "vertical-stack"` |

A workspace set to **Default layout** has no entry in the file, so it follows
`layout.default`.

## Borders

| Section | Setting | Key in `config.toml` |
|---------|---------|----------------------|
| Border | Width, corners | `borders.width`, `borders.corner_style` |
| Colors | Focused, monocle, unfocused | `borders.colors.focused`, `borders.colors.monocle`, `borders.colors.unfocused` |

Each color is picked from a swatch. Click it to choose the theme default, a
named color of the theme being edited (the palette changes with the theme), or
type a `#RRGGBB` value; a hex that isn't a color is not taken. Unfocused also
offers None, which draws no border on windows without focus.

## Mouse

| Section | Setting | Key in `config.toml` |
|---------|---------|----------------------|
| Focus | Pointer follows focus, focus follows pointer | `mouse.follows_focus`, `mouse.focus_follows_mouse` |
| Dragging | Drag to swap, resize splits | `mouse.drag_to_swap`, `mouse.resize_splits` |

## Animations

| Section | Setting | Key in `config.toml` |
|---------|---------|----------------------|
| Motion | Animate windows, duration, easing | `animations.enabled`, `animations.duration`, `animations.easing` |
| System | Follow the system | `animations.follow_system` |

Easing lists the presets and **Custom curve**, which shows a field for a
`cubic-bezier(x1, y1, x2, y2)` curve. A curve that is not valid says why under
the setting's name, and Apply refuses it until it is fixed.

## Keybindings

Edits `keybindings.toml`. The hotkeys are grouped by what they do: focus,
moving windows, workspaces, layout, floating windows and so on. Each row
shows the action and its key combination as keycaps.

Click **Change** on a row, then press the new combination. Hold at least one
of Ctrl, Alt or Shift (function keys can stand alone). The first key you
press is taken; **Esc** cancels. A key mosaico has no name for is refused and
the row keeps waiting. A binding that is not on its default shows the default
and a reset button.

If two bindings share a combination, in any order or case of its modifiers,
both rows are marked **Conflict** and name each other. **Apply** will not
save until one of them is changed. Search matches the action and the keys,
so `alt+j` or `float` narrows the list.

Applying changes only the keys you edited: comments, blank lines, the order
and the line endings of `keybindings.toml` are kept. The daemon picks the new
keys up as it does when you edit the file by hand.

## Rules

The **Rules** page edits `user-rules.toml`. Each of your rules is a card with
the window class, a part of the title and the program path to match (leave a
field empty to match anything), and what to do with the window: tile it,
float it or ignore it. **Up** and **Down** move a rule, because the first
rule that matches a window decides, and **Delete** removes it. **Add rule**
adds one at the end; a rule needs at least a class, a title or a path before
it can be saved.

Apply keeps the comments and blank lines of the file, and a rule that moves
takes its comment along. Your rules match before the community rules in
`rules.toml`, which are listed under **Community rules** and cannot be edited
here. Searching filters both lists by pattern or action.

## Current Limits

Pages that are not ready yet are grayed out in the sidebar. Edit the
[configuration files](configuration.md) to change those settings for now.

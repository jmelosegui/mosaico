# Settings Window

The settings window lets you change mosaico without editing TOML files. It is a
separate program, `mosaico-settings.exe`, installed beside `mosaico.exe`.

## Opening It

- Run `mosaico settings` from a terminal.
- Double-click `mosaico-settings.exe` in the install folder.

The daemon does not need to be running, and starting the daemon does not open
the window.

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
setting you have changed. The footer counts your unsaved changes, and
**Revert** drops them all.

## General

| Section | Setting | File |
|---------|---------|------|
| Appearance | Theme and flavor | `config.toml` |
| Startup | Start at sign-in | Registry |
| Logging | Write a log file, level, maximum file size | `config.toml` |

Picking a theme or flavor previews it in the settings window right away.

## Current Limits

This release has the General page only; the other pages appear in the sidebar
but are not ready yet. **Apply** is disabled, so changes made here are not
saved. Edit the [configuration files](configuration.md) to change settings for
now.

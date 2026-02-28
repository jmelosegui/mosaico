# Getting Started

This guide walks you through your first session with Mosaico.

## 1. Generate Configuration Files

Create the default configuration files in `~/.config/mosaico/`:

```sh
mosaico init
```

This generates three files:

| File | Purpose |
|------|---------|
| `config.toml` | Layout, border, theme, and logging settings |
| `keybindings.toml` | Keyboard shortcut mappings |
| `rules.toml` | Rules for excluding windows from tiling |

Existing files are never overwritten, so it is safe to run `init` again.

## 2. Start the Window Manager

```sh
mosaico start
```

Mosaico launches a background daemon that immediately tiles all visible
windows on your desktop. You will see a startup banner with your config
directory, the daemon PID, and a helpful tip.

## 3. Navigate with Keyboard Shortcuts

The default keybindings use vim-style motions:

| Shortcut | Action |
|----------|--------|
| `Alt + H` | Focus the window to the left |
| `Alt + J` | Focus the window below |
| `Alt + K` | Focus the window above |
| `Alt + L` | Focus the window to the right |
| `Alt + Shift + H` | Move window left |
| `Alt + Shift + J` | Move window down |
| `Alt + Shift + K` | Move window up |
| `Alt + Shift + L` | Move window right |
| `Alt + Shift + R` | Re-apply the tiling layout |
| `Alt + T` | Toggle monocle (full-screen) mode |
| `Alt + Q` | Close the focused window |
| `Alt + 1-8` | Switch to workspace 1-8 |
| `Alt + Shift + 1-8` | Send focused window to workspace 1-8 |

## 4. Check the Status

```sh
mosaico status
```

Reports whether the daemon is running and its PID.

## 5. Run the Doctor

```sh
mosaico doctor
```

Performs a health check on your configuration files, daemon status, and
monitor setup. Any issues are reported with colored status tags.

## 6. Stop the Window Manager

```sh
mosaico stop
```

All windows are restored to their original state before the daemon exits.

## Next Steps

- [CLI Commands](cli.md) -- full command reference
- [Configuration](configuration.md) -- customize layout, borders, and more
- [Keyboard Bindings](keybindings.md) -- change or add shortcuts
- [Workspaces](workspaces.md) -- learn about multi-workspace support

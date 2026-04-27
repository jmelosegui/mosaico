# Getting Started

This guide walks you through your first session with Mosaico.

## 1. Generate Configuration Files

Create the default configuration files in `~/.config/mosaico/`:

```sh
mosaico init
```

This generates the following files:

| File | Purpose |
|------|---------|
| `config.toml` | Layout, border, theme, and logging settings |
| `keybindings.toml` | Keyboard shortcut mappings |
| `rules.toml` | Community rules (auto-updated on daemon startup) |
| `user-rules.toml` | Personal rule overrides (never overwritten) |
| `bar.toml` | Status bar settings |

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
| `Alt + H/J/K/L` | Focus left/down/up/right |
| `Alt + Shift + H/J/K/L` | Move or swap the focused window |
| `Alt + 1` .. `Alt + 8` | Switch to workspace 1-8 |
| `Alt + Shift + 1` .. `Alt + Shift + 8` | Send focused window to workspace 1-8 |
| `Alt + N` | Cycle layout (BSP / VerticalStack / ThreeColumn) |
| `Alt + T` | Toggle monocle (full-screen) mode |
| `Alt + M` | Minimize the focused window |
| `Alt + Q` | Close the focused window |
| `Alt + Shift + R` | Retile (re-apply the current layout) |
| `Alt + Shift + P` | Pause / resume all Mosaico hotkeys |

The full reference card lives at the [Cheatsheet](cheatsheet.md). Edit
`~/.config/mosaico/keybindings.toml` to remap any shortcut.

## 4. Pick a Theme (optional)

Mosaico ships with three theme families that color the status bar and
the focus border. Set the active theme in `~/.config/mosaico/config.toml`:

```toml
[theme]
name = "rose-pine"   # catppuccin (default), rose-pine, tokyo-night
flavor = "main"
```

Save and the colors update live, no restart needed. See
[Theming](theming.md) for the full palette per family.

## 5. Check the Status

```sh
mosaico status
```

Reports whether the daemon is running and its PID.

## 6. Run the Doctor

```sh
mosaico doctor
```

Performs a health check on your configuration files, daemon status, and
monitor setup. Any issues are reported with colored status tags.

## 7. Stop the Window Manager

```sh
mosaico stop
```

All windows are restored to their original state before the daemon exits.

## Next Steps

- [Cheatsheet](cheatsheet.md) -- one-page reference card for every default shortcut
- [CLI Commands](cli.md) -- full command reference
- [Configuration](configuration.md) -- customize layout, borders, and more
- [Keyboard Bindings](keybindings.md) -- change or add shortcuts
- [Workspaces](workspaces.md) -- per-monitor and global workspace modes
- [Focus Borders](focus-border.md) -- focused, monocle, and unfocused border colors
- [Theming](theming.md) -- Catppuccin, Rosé Pine, Tokyo Night palettes

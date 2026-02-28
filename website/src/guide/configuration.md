# Configuration

Mosaico uses TOML configuration files stored in `~/.config/mosaico/`. All
settings have sensible defaults, so configuration files are entirely optional.

Run `mosaico init` to generate fully-commented template files.

## Configuration Files

| File | Purpose | Hot-Reload |
|------|---------|------------|
| `config.toml` | Layout, borders, theme, logging | Yes |
| `keybindings.toml` | Keyboard shortcuts | No (restart required) |
| `rules.toml` | Window management rules | Yes |
| `bar.toml` | Status bar settings | Yes |

## `config.toml`

Controls layout, borders, logging, and theme:

```toml
[layout]
gap = 8       # Pixel gap between windows (0-200)
ratio = 0.5   # BSP split ratio (0.1-0.9)

[borders]
width = 4              # Border thickness in pixels (0-32)
focused = "#00b4d8"    # Hex color for focused window
monocle = "#2d6a4f"    # Hex color for monocle mode

[theme]
flavor = "mocha"   # Catppuccin flavor: latte, frappe, macchiato, mocha

[logging]
enabled = false    # Enable file logging
level = "info"     # Log level: debug, info, warn, error
max_file_mb = 10   # Max log file size before rotation
```

## `keybindings.toml`

Maps key combinations to actions. See
[Keyboard Bindings](keybindings.md) for the full reference.

```toml
[[keybinding]]
action = "focus-down"
key = "J"
modifiers = ["alt"]
```

## `bar.toml`

Configures the status bar. See [Status Bar](status-bar.md) for details.

```toml
enabled = true
height = 28
monitor = "all"      # "all", "primary", or 0-based index

[colors]
background = "base"      # Named Catppuccin color
foreground = "text"
accent = "blue"

[[left]]
type = "workspaces"

[[center]]
type = "clock"
format = "%H:%M"

[[right]]
type = "cpu"
```

## `rules.toml`

Controls which windows are managed by the tiler. See
[Window Rules](window-rules.md) for details.

```toml
[[rule]]
match_class = "ApplicationFrameWindow"
manage = false
```

## Partial Files

All configuration fields have defaults. You only need to include the
settings you want to change. A file containing only:

```toml
[layout]
gap = 12
```

will use defaults for everything else.

## Validation

Out-of-range values are automatically clamped to safe boundaries rather
than rejected:

| Setting | Valid Range |
|---------|------------|
| `gap` | 0 -- 200 |
| `ratio` | 0.1 -- 0.9 |
| `border width` | 0 -- 32 |

## Hot-Reload

Changes to `config.toml`, `rules.toml`, and `bar.toml` are automatically
detected and applied while the daemon is running (polled every 2 seconds).

- **config.toml** -- layout gap/ratio, border settings, and theme are
  reloaded. Windows are retiled immediately.
- **rules.toml** -- new rules apply to newly created windows. Existing
  managed windows are not re-evaluated.
- **bar.toml** -- the status bar is recreated with updated settings and
  colors.
- **keybindings.toml** -- **not** hot-reloaded. Changes require a daemon
  restart (`mosaico stop && mosaico start`).

Only valid configurations are applied. If a file change introduces a parse
error, the current configuration is kept and a warning is logged.

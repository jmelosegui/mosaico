# Mosaico

A fast, configurable tiling window manager for Windows, written in Rust.

<img alt="image" src="https://github.com/user-attachments/assets/13b02a9c-cbd5-4fc7-9498-f3399556fc7d" />

**Documentation:** [jmelosegui.com/mosaico](https://jmelosegui.com/mosaico/)

## Features

- Automatic tiling with three layout algorithms (BSP, VerticalStack,
  ThreeColumn) cyclable per workspace at runtime
- Up to 8 workspaces with per-monitor (default) or global switching, the
  latter mirroring Windows virtual desktops
- Monocle mode for distraction-free single-window focus
- Vim-style keybindings out of the box
- Hot-reload for config and window rules — no restart needed
- Pause/unpause all hotkeys with `Alt + Shift + P` for full-screen apps
- Per-app rules to exclude windows from tiling
- Colored window borders to identify the focused window
- Status bar with workspace indicators, clock, CPU, RAM, and more
- Multi-monitor support with cross-monitor focus and window movement
- One-line install, zero dependencies
- Built-in `doctor` command to diagnose setup issues

## Installation

### Windows

```powershell
irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex
```

### From source

```sh
cargo install --path .
```

## Usage

```sh
mosaico start              # Start the window manager
mosaico stop               # Stop the window manager
mosaico status             # Show current status
mosaico doctor             # Check your setup for common issues
mosaico init               # Create default configuration files
mosaico autostart enable   # Start automatically on Windows boot
mosaico autostart disable  # Remove automatic startup
mosaico autostart status   # Check if autostart is enabled
mosaico --help             # Show all available commands
```

### Actions

```sh
mosaico action focus left         # Focus window to the left
mosaico action focus right        # Focus window to the right
mosaico action focus up           # Focus window above
mosaico action focus down         # Focus window below
mosaico action move left          # Move window to the left
mosaico action move right         # Move window to the right
mosaico action move up            # Move window up
mosaico action move down          # Move window down
mosaico action retile             # Re-apply the current layout
mosaico action toggle-monocle     # Toggle monocle mode
mosaico action cycle-layout       # Cycle BSP / VerticalStack / ThreeColumn
mosaico action close-focused      # Close the focused window
mosaico action minimize-focused   # Minimize the focused window
mosaico action goto-workspace-3   # Switch to workspace 3 (1-8)
mosaico action send-to-workspace-5  # Send focused window to workspace 5 (1-8)
mosaico pause                     # Pause all hotkeys (also Alt + Shift + P)
mosaico unpause                   # Resume all hotkeys
```

### Debugging

```sh
mosaico debug list    # List all visible windows
mosaico debug events  # Watch window events in real time
```

## Configuration

Configuration files live in `~/.config/mosaico/`:

| File | Purpose |
|------|---------|
| `config.toml` | Layout (gap, ratio), workspace mode, borders, theme, logging |
| `keybindings.toml` | Hotkey mappings (vim-style by default) |
| `bar.toml` | Status bar widgets and colors |
| `rules.toml` | Community window rules (auto-downloaded) |
| `user-rules.toml` | Personal rule overrides; take priority over `rules.toml` |

Changes to `config.toml`, `bar.toml`, and `user-rules.toml` are
hot-reloaded automatically. Keybinding changes require a daemon restart.

Run `mosaico init` to create the default configuration files.

### Default keybindings

| Shortcut             | Action                            |
|----------------------|-----------------------------------|
| Alt + H              | Focus left                        |
| Alt + J              | Focus down                        |
| Alt + K              | Focus up                          |
| Alt + L              | Focus right                       |
| Alt + Shift + H      | Move left                         |
| Alt + Shift + J      | Move down                         |
| Alt + Shift + K      | Move up                           |
| Alt + Shift + L      | Move right                        |
| Alt + Shift + R      | Retile                            |
| Alt + T              | Toggle monocle                    |
| Alt + N              | Cycle layout (BSP / VStack / 3Col) |
| Alt + Q              | Close focused                     |
| Alt + M              | Minimize focused                  |
| Alt + 1 .. Alt + 8       | Switch to workspace 1-8           |
| Alt + Shift + 1 .. Alt + Shift + 8 | Send focused window to workspace 1-8 |
| Alt + Shift + P      | Pause / unpause all hotkeys       |

### Logging

Logging is opt-in. Enable it in `config.toml`:

```toml
[logging]
enabled = true
level = "info"        # debug, info, warn, error
max_file_mb = 10
```

Logs are written to `~/.config/mosaico/logs/mosaico.log` with automatic rotation.

## Building from source

```sh
cargo build --release
```

## Running tests

```sh
cargo test
```

## License

[MIT](LICENSE)

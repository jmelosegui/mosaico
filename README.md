# Mosaico

A cross-platform tiling window manager, written in Rust.

## Installation

### Windows

```powershell
irm https://mosaico.dev/install.ps1 | iex
```

### From source

```sh
cargo install --path .
```

## Usage

```sh
mosaico start    # Start the window manager
mosaico stop     # Stop the window manager
mosaico status   # Show current status
mosaico doctor   # Check your setup for common issues
mosaico init     # Create default configuration files
mosaico --help   # Show all available commands
```

### Actions

```sh
mosaico action focus left      # Focus window to the left
mosaico action focus right     # Focus window to the right
mosaico action focus up        # Focus window above
mosaico action focus down      # Focus window below
mosaico action move left       # Move window to the left
mosaico action move right      # Move window to the right
mosaico action move up         # Move window up
mosaico action move down       # Move window down
mosaico action retile          # Re-apply the current layout
mosaico action toggle-monocle  # Toggle monocle mode
mosaico action close-focused   # Close the focused window
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
| `config.toml` | Layout (gap, ratio) and border settings |
| `keybindings.toml` | Hotkey mappings (vim-style by default) |
| `rules.toml` | Window rules for excluding apps from tiling |

Changes to `config.toml` and `rules.toml` are hot-reloaded automatically.
Keybinding changes require a daemon restart.

Run `mosaico init` to create the default configuration files.

### Default keybindings

| Shortcut        | Action         |
|-----------------|----------------|
| Alt + H         | Focus left     |
| Alt + J         | Focus down     |
| Alt + K         | Focus up       |
| Alt + L         | Focus right    |
| Alt + Shift + H | Move left      |
| Alt + Shift + J | Move down      |
| Alt + Shift + K | Move up        |
| Alt + Shift + L | Move right     |
| Alt + Shift + R | Retile         |
| Alt + T         | Toggle monocle |
| Alt + Q         | Close focused  |

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

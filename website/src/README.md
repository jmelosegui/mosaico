# Mosaico

A cross-platform tiling window manager, written in Rust.

Mosaico automatically arranges your windows into a non-overlapping tiled
layout using a Binary Space Partitioning (BSP) algorithm. It runs as a
lightweight background daemon and is controlled entirely from the command
line or via global keyboard shortcuts.

## Features

- **Automatic tiling** -- windows are arranged in a BSP layout the moment
  they open, close, or are moved between monitors.
- **Vim-style navigation** -- focus and move windows with `Alt + H/J/K/L`.
- **Workspaces** -- up to 8 independent workspaces per monitor.
- **Multi-monitor** -- each monitor is managed independently with
  cross-monitor navigation.
- **Focus borders** -- colored overlay border around the focused window.
- **Monocle mode** -- full-screen single-window mode per monitor.
- **Status bar** -- configurable per-monitor bar with workspace indicators,
  clock, CPU, RAM, and more.
- **Catppuccin theming** -- built-in theme system with four flavors.
- **Hot-reload** -- configuration changes are applied without restarting.
- **Window rules** -- exclude specific applications from tiling.

## Quick Start

```powershell
# Install on Windows
irm https://raw.githubusercontent.com/jmelosegui/mosaico/main/docs/install.ps1 | iex

# Create default config files
mosaico init

# Start the window manager
mosaico start
```

See [Installation](guide/installation.md) for more options or
[Getting Started](guide/getting-started.md) for a walkthrough.

## License

Mosaico is released under the [MIT License](https://github.com/jmelosegui/mosaico/blob/main/LICENSE).

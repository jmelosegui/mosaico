# Mosaico

A cross-platform tiling window manager, written in Rust.

<img alt="Mosaico screenshot" src="https://github.com/user-attachments/assets/13b02a9c-cbd5-4fc7-9498-f3399556fc7d" />

Mosaico automatically arranges your windows into a non-overlapping tiled
layout. It ships three tiling algorithms (BSP, VerticalStack, ThreeColumn)
that you can cycle per workspace at runtime, runs as a lightweight
background daemon, and is controlled entirely from the command line or
via global keyboard shortcuts.

## Features

- **Automatic tiling with three algorithms** -- BSP (default),
  VerticalStack, and ThreeColumn. Cycle on the fly with `Alt + N` or
  pin a layout per workspace.
- **Vim-style navigation** -- focus and move windows with `Alt + H/J/K/L`.
- **Workspaces** -- up to 8 workspaces, switching either per-monitor
  (default) or globally across every display, like Windows virtual desktops.
- **Multi-monitor** -- each monitor is managed independently with
  cross-monitor navigation.
- **Per-window borders** -- focused, monocle, and unfocused color states
  under `[borders.colors]`.
- **Monocle mode** -- full-screen single-window mode per monitor.
- **Status bar** -- configurable per-monitor bar with workspace indicators,
  clock, CPU, RAM, focused window icon, pause indicator, and more.
- **Three theme families** -- Catppuccin, Rosé Pine, and Tokyo Night
  with multiple flavors each.
- **Pause / unpause hotkeys** -- `Alt + Shift + P` releases every Mosaico
  shortcut so full-screen applications can claim the same combinations.
- **Hot-reload** -- changes to `config.toml`, `bar.toml`, and
  `user-rules.toml` are applied without restarting.
- **Window rules** -- community-maintained rules plus user overrides to
  exclude specific applications from tiling.

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

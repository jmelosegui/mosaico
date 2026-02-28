# Mosaico — Development Plan

A cross-platform tiling window manager, written in Rust.

## Architecture

Mosaico is structured as a Cargo workspace with multiple crates:

- **mosaico** — the CLI binary
- **mosaico-core** — platform-agnostic traits and types
- **mosaico-windows** — Windows platform implementation
- Future: `mosaico-linux`, `mosaico-macos`

## Phases

| Phase | Description | Status |
|-------|-------------|--------|
| [1](phase-01.md) | Project Initialization | Complete |
| [2](phase-02.md) | CLI with clap | Complete |
| [3a](phase-03a.md) | Restructure into Cargo Workspace | Complete |
| [3b](phase-03b.md) | Window Enumeration (`mosaico debug list`) | Complete |
| [4](phase-04.md) | Daemon + IPC | Complete |
| [5](phase-05.md) | Event-Driven Window Discovery | Complete |
| [6](phase-06.md) | Window Positioning | Complete |

| [7](phase-07.md) | Tiling Layouts | Complete |
| [8](phase-08.md) | Hotkey System | Complete |
| [9](phase-09.md) | Configuration | Complete |
| [10](phase-10.md) | Multi-Monitor Support | Complete |
| [11](phase-11.md) | Window Rules | Complete |
| [12](phase-12.md) | Focus Borders, Monocle Mode, Vim Motions | Complete |
| [13](phase-13.md) | Unified Navigation, Doctor, Banner, Hot-Reload | Complete |
| [14](phase-14.md) | Status Bar | Complete |
| [15](phase-15.md) | Catppuccin Theming | Complete |
| [16](phase-16.md) | Active Window Icon Widget | Complete |
| [17](phase-17.md) | Community Rules Repository | Complete |
| [18](phase-18.md) | Auto-Start on Windows Boot | Complete |
| [19](phase-19.md) | Media Widget | Planned |

## Design Principles

- No source file should exceed ~150 lines
- One concern per file
- Platform-specific code lives in its own crate, never in `mosaico-core`
- `mosaico-core` defines traits; platform crates implement them

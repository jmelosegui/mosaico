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
| [1](phase-01.md) | Project Initialization | In Progress |
| [2](phase-02.md) | CLI with clap | Complete |
| [3a](phase-03a.md) | Restructure into Cargo Workspace | Complete |
| [3b](phase-03b.md) | Window Enumeration (`mosaico debug list`) | Complete |

## Future Phases (not yet planned in detail)

4. **Event-driven window discovery** — `SetWinEventHook` for runtime window events
5. **Window positioning** — Move/resize windows using Win32 API
6. **Tiling layouts** — BSP or master-stack layout algorithms
7. **Hotkey system** — Global hotkey registration for keyboard-driven control
8. **Configuration** — TOML/YAML config file for user preferences
9. **Multi-monitor support**
10. **Window rules** — Per-application tiling behavior
11. **IPC** — Named pipe or socket for daemon communication

## Design Principles

- No source file should exceed ~150 lines
- One concern per file
- Platform-specific code lives in its own crate, never in `mosaico-core`
- `mosaico-core` defines traits; platform crates implement them

# Phase 13: Unified Navigation, Doctor, Banner, and Hot-Reload

**Status:** Complete

**Goal:** Unify spatial navigation under a single Direction enum, add a `doctor`
command for health checks, redesign the startup banner, and implement
hot-reloading for configuration files.

## Unified Spatial Navigation

Merged `find_neighbor` and `find_vertical_neighbor` into a single
`find_neighbor(Direction)` function in `spatial.rs`. The algorithm:

1. Filter candidates in the requested direction with perpendicular overlap > 0
2. Pick closest by edge distance (gap between touching edges)
3. Tiebreaker: topmost (for horizontal) or leftmost (for vertical)

Monitor overflow is horizontal-only: when `find_neighbor` returns None for
left/right, `resolve_horizontal_target` checks for a physically adjacent
monitor. Up/down never cross monitors.

### Action/Direction redesign

```rust
pub enum Direction { Left, Right, Up, Down }
pub enum Action { Focus(Direction), Move(Direction), Retile, ToggleMonocle, CloseFocused }
```

Serde support via `#[serde(try_from = "String", into = "String")]` with
`FromStr`/`Display` using prefix parsing (`"focus-left"`, `"move-down"`).

## Doctor Command

Seven health checks with auto-healing:

- Config directory exists (auto-creates if missing)
- `config.toml` parses correctly
- `keybindings.toml` parses correctly
- Keybinding keys are valid Win32 key names
- `rules.toml` parses correctly
- Daemon is running (auto-removes stale PID files)
- Monitors are detected

Status markers: `[ok]`, `[warn]`, `[fail]`, `[fixed]`

## Start Banner

ASCII art "MOSAICO" logo in bright blue with version right-aligned.
Info lines show config path, daemon PID, repo URL, and a rotating tip
(8 tips, rotated by epoch seconds to avoid Windows PID bias).

## Configuration Hot-Reload

`config_watcher.rs` polls file modification times every 2 seconds. Changes
are validated with `try_load`/`try_load_rules` before applying. Invalid
changes are logged and skipped. Keybinding hot-reload was deferred (requires
Win32 event loop thread coordination).

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      action.rs             # Direction enum, Action with Focus(Direction)/Move(Direction)
      spatial.rs             # NEW: find_neighbor(Direction), find_entry(Direction), 35 tests
      rect.rs               # Added horizontal_overlap()
      workspace.rs           # Added insert(index, hwnd)
      config/
        loader.rs           # Added try_load, try_load_keybindings, try_load_rules
        mod.rs              # Re-exports try_ variants
        keybinding.rs       # Updated defaults for Direction
        template.rs         # Updated TOML template
  mosaico-windows/
    src/
      config_watcher.rs     # NEW: file modification polling + validated reload
      daemon.rs             # Config watcher thread, DaemonMsg::Reload
      tiling.rs             # reload_config(), reload_rules(), unified focus/move
  mosaico/
    src/
      main.rs               # Doctor subcommand, nested focus/move with DirectionCommands
      commands/
        banner.rs           # NEW: shared ASCII art logo
        doctor.rs           # NEW: health checks with auto-healing
        start.rs            # Redesigned banner with tips
```

## Tasks

- [x] Create Direction enum and refactor Action to use Focus(Direction)/Move(Direction)
- [x] Implement unified find_neighbor(Direction) in spatial.rs with 35 tests
- [x] Add horizontal_overlap to Rect, insert(index) to Workspace
- [x] Update CLI to use nested focus/move subcommands with DirectionCommands
- [x] Add try_load variants to config loader for error reporting
- [x] Implement doctor command with 7 checks and auto-healing
- [x] Redesign start banner with ASCII art, tips, and repo link
- [x] Implement config_watcher with 2-second polling and validated reload
- [x] Integrate config watcher into daemon with DaemonMsg::Reload
- [x] Build, lint, test
- [x] Commit

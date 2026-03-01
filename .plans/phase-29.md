# Phase 29: Toggle Pause (Stop/Resume Daemon)

**Status:** Pending

**Goal:** Add a single command that pauses the daemon (stops managing
windows) and can be resumed later, effectively a toggle.

## Overview

Users sometimes want to temporarily disable Mosaico without fully
shutting down the process. This phase adds a `toggle-pause` command that
switches the daemon between **active** and **paused** states. When
paused, Mosaico removes its effects (borders, tiling, hooks) but keeps
the daemon ready to resume quickly.

## User Experience

```
$ mosaico toggle-pause
Paused Mosaico (daemon still running).

$ mosaico toggle-pause
Resumed Mosaico.
```

## Behavior

- **Pause**:
  - Stop applying layout changes.
  - Restore all managed windows (show/uncloak, reset borders, clear
    topmost/focus overlays).
  - Unregister hotkeys (so Mosaico doesn't intercept input).
  - Keep IPC server and daemon thread alive.

- **Resume**:
  - Re-register hotkeys.
  - Re-enumerate windows and re-apply layout.
  - Re-enable borders and any active widgets.

## Architecture

### New State Flag

Add a paused flag to the daemon state (e.g., `TilingManager` or daemon
loop state):

```rust
pub struct TilingManager {
    // ... existing fields ...
    paused: bool,
}
```

### IPC Action

Add a new action in `mosaico-core`:

```rust
pub enum Action {
    // ... existing ...
    TogglePause,    // toggle-pause
}
```

### Daemon Handling

Implement `toggle_pause()` in `mosaico-windows`:

- If `paused == false`: perform pause sequence and set `paused = true`.
- If `paused == true`: perform resume sequence and set `paused = false`.

Ensure pause/resume is idempotent and safe if called repeatedly.

### CLI

Add a `toggle-pause` subcommand to `mosaico` CLI which sends the IPC
action to the running daemon.

## Edge Cases

1. **Daemon not running**: CLI prints a clear error and exits 1.
2. **Paused + config reload**: store changes but do not retile until
   resumed.
3. **Paused + workspace switch**: treat as no-op (or ignore requests).
4. **Paused + monocle**: exit monocle on pause, restore on resume.

## Modified Files

```
crates/
  mosaico-core/
    src/
      action.rs              # Add TogglePause
  mosaico-windows/
    src/
      daemon.rs              # Handle pause/resume state
      tiling/mod.rs          # toggle_pause() implementation
      hotkey.rs              # unregister/register for pause/resume
  mosaico/
    src/
      main.rs                # Add toggle-pause CLI subcommand
      commands/action.rs     # Route toggle-pause
```

## Tasks

- [ ] Add `TogglePause` to `Action`
- [ ] Add `toggle-pause` parsing/display strings
- [ ] Add `toggle_pause()` handling in daemon
- [ ] Add paused state flag to daemon state
- [ ] Implement pause: unregister hotkeys, restore windows, clear borders
- [ ] Implement resume: re-register hotkeys, re-enumerate, re-tile
- [ ] Add CLI subcommand and IPC wiring
- [ ] Add integration test: toggle pause twice restores state
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: pause stops tiling and hotkeys
- [ ] Manual test: resume restores tiling and hotkeys
- [ ] Update `.plans/plan.md`

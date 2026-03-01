# Phase 26: Floating Window Toggle

**Status:** Pending

**Goal:** Allow users to toggle any managed window between tiled and
floating states with a keyboard shortcut.

## Overview

Some windows don't belong in a tiling layout — media players, reference
images, calculator, small utilities, chat popups. Currently these must
either be excluded entirely via window rules (and then Mosaico ignores
them completely) or forced into the tiling grid where they waste space.

This phase adds a floating layer: windows toggled to float are removed
from the tiling layout but remain managed by Mosaico. They keep their
current size and position (or snap to a centered default) and hover
above the tiled windows. Focus navigation skips floating windows unless
the user explicitly focuses them.

## Reference Design

```
# Normal tiled layout:
┌──────────┬──────────┐
│          │          │
│  Browser │ Terminal │
│          │          │
└──────────┴──────────┘

# After Alt+F on Terminal (floats it):
┌─────────────────────┐
│                     │
│  Browser (fills)    │   ┌─────────┐
│                     │   │ Terminal │ (floating, centered)
│                     │   │ (float) │
└─────────────────────┘   └─────────┘

# After Alt+F again on Terminal (re-tiles it):
┌──────────┬──────────┐
│          │          │
│  Browser │ Terminal │
│          │          │
└──────────┴──────────┘
```

## Configuration

### Default Keybinding

```toml
# Toggle floating: Alt + F
[[keybinding]]
action = "toggle-float"
key = "F"
modifiers = ["alt"]
```

`Alt+F` is intuitive (F for float) and currently unbound.

## Architecture

### Floating Layer Concept

Each workspace gains a floating set alongside its tiled handle list:

```rust
pub struct Workspace {
    handles: Vec<usize>,              // tiled windows
    split_ratios: HashMap<...>,       // per-split ratios (phase 25)
    floating: HashSet<usize>,         // floating window handles
}
```

A floating window is:
- **Tracked** by the workspace (moves with workspace switches)
- **Not included** in layout computation
- **Shown/hidden** during workspace switches (same as tiled windows)
- **Above** tiled windows in Z-order

### Toggle Float Flow

When the user presses `Alt+F` on the focused window:

1. **Tiled -> Float**:
   - Remove hwnd from `handles` (tiled list)
   - Add hwnd to `floating` set
   - Re-tile the workspace (remaining windows fill the gap)
   - Set the floating window's Z-order above tiled windows
   - Keep current position/size (don't move it)
   - Update border to show floating state

2. **Float -> Tiled**:
   - Remove hwnd from `floating` set
   - Add hwnd back to `handles` (at the end)
   - Re-tile the workspace (window joins the layout)
   - Update border

### Focus Behavior

Floating windows need special focus handling:

- `Alt+H/J/K/L` (directional focus): Only navigates tiled windows.
  Floating windows are skipped in spatial navigation.
- `Alt+F` on a tiled window while a float exists: Focuses the float?
  No — `Alt+F` toggles the focused window's float state.
- **Clicking a floating window**: `EVENT_SYSTEM_FOREGROUND` fires,
  Mosaico updates `focused_window` normally.
- **Focus cycle** (future): Could add `cycle-focus` that includes
  floating windows.

For now, floating windows are focused via mouse click or Alt-Tab. This
is the simplest correct behavior and matches user expectations — floats
are "out of the grid" and managed independently.

### Z-Order

When a window is toggled to float, call `SetWindowPos` with
`HWND_TOPMOST` to keep it above tiled windows. When toggled back to
tiled, reset to `HWND_NOTOPMOST`.

```rust
// Float: stay on top
SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0,
    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);

// Unfloat: normal Z-order
SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0,
    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
```

### Workspace Switching

Floating windows must be hidden/shown along with tiled windows during
workspace switches. In `goto_workspace()` and `send_to_workspace()`,
iterate both `handles` and `floating`:

```rust
for &hwnd in mon.active_ws().all_handles() {
    self.hide_window(hwnd);
}
```

Add `all_handles()` to `Workspace` that chains tiled + floating handles.

### Window Destruction

When a floating window is destroyed (`EVENT_OBJECT_DESTROY`), remove it
from the `floating` set. No re-tiling needed since it wasn't in the
layout.

### Monocle Interaction

- If the focused window is floating and monocle is toggled: the float
  becomes the monocle window (unlinked from floating, fills monitor).
- If monocle is active and the user toggles float: exit monocle first,
  then float.

### Border Colors

Floating windows could use a distinct border color to visually
differentiate them from tiled windows. This can be added in a future
phase — for now, use the standard focused/unfocused border.

### Bar Display

The layout widget could show a float indicator (e.g., `BSP | F` when
floating windows exist). Simple addition to `bar_states()`.

### New Action

```rust
pub enum Action {
    // ... existing ...
    ToggleFloat,    // toggle-float
}
```

### Send-to-Workspace with Floats

When a floating window is sent to another workspace:
- Remove from source `floating` set
- Add to target `floating` set
- Hide/show as part of the workspace switch

### Restore All Windows

`restore_all_windows()` (daemon shutdown) must iterate floating windows
too, resetting Z-order and uncloaking/showing them.

## Modified Files

```
crates/
  mosaico-core/
    src/
      action.rs              # Add ToggleFloat
      workspace.rs           # Add floating set, all_handles(), toggle methods
      config/
        keybinding.rs        # Add Alt+F default keybinding
        template.rs          # Add toggle-float to keybindings template
  mosaico-windows/
    src/
      tiling/
        mod.rs               # Handle ToggleFloat action, update event handlers
        layout.rs            # toggle_float() implementation
        workspace.rs         # Hide/show floating windows during switches
      window.rs              # Add set_topmost() / clear_topmost() methods
  mosaico/
    src/
      main.rs                # Add ToggleFloat CLI subcommand
```

## Edge Cases

1. **All windows floating**: Workspace has no tiled windows. Layout
   produces no positions. Focus goes to a floating window. No border
   issues (border tracks focused window regardless of state).

2. **Float + monocle**: If monocle is active and user floats the
   monocle window, exit monocle mode. The window becomes floating
   and other windows re-tile.

3. **Float + minimize**: Minimizing a floating window fires
   `EVENT_SYSTEM_MINIMIZESTART`. Remove from `floating` set (same as
   tiled minimize removes from `handles`).

4. **Window rules + float**: If a rule says `manage = false`, the
   window is never tracked at all. Float only applies to managed
   windows. Future: add `float = true` rule option for auto-float.

5. **Cross-monitor float**: Moving a floating window to another monitor
   via `move-left`/`move-right` moves it to the other monitor's
   floating set. The window keeps its floating state.

6. **Resize floating window**: The user can resize floating windows
   with the mouse (Windows native resize). Mosaico doesn't interfere
   since it's not in the layout.

## Tasks

- [ ] Add `floating: HashSet<usize>` to `Workspace` struct
- [ ] Add `is_floating()`, `toggle_float()`, `all_handles()` to `Workspace`
- [ ] Add `ToggleFloat` variant to `Action` enum
- [ ] Update `Action::from_str` / `Display` for `toggle-float`
- [ ] Add default keybinding: Alt+F -> ToggleFloat
- [ ] Add `toggle-float` to keybindings template
- [ ] Implement `toggle_float()` in `TilingManager`:
  - [ ] Tiled -> Float: remove from handles, add to floating, retile,
        set Z-order topmost
  - [ ] Float -> Tiled: remove from floating, add to handles, retile,
        clear Z-order
- [ ] Add `set_topmost()` and `clear_topmost()` to `Window`
- [ ] Update workspace switching to hide/show floating windows
- [ ] Update `Destroyed` event handler to check floating set
- [ ] Update `Minimized` event handler to check floating set
- [ ] Update `restore_all_windows()` to restore floating windows
- [ ] Update `bar_states()` to indicate floating windows exist
- [ ] Skip floating windows in spatial focus/move navigation
- [ ] Handle monocle + float interaction
- [ ] Add `ToggleFloat` subcommand to CLI
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: toggle window to float, verify it leaves layout
- [ ] Manual test: toggle float back to tiled, verify it rejoins layout
- [ ] Manual test: workspace switch with floating windows
- [ ] Manual test: close a floating window
- [ ] Manual test: monocle + float interaction
- [ ] Manual test: all windows floating (empty tiled layout)
- [ ] Update documentation (`docs/tiling-layout.md`, `docs/configuration.md`)
- [ ] Update `.plans/plan.md`

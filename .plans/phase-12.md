# Phase 12: Focus Borders, Monocle Mode, and Vim Motions

**Status:** Complete

**Goal:** Add visual focus indicators (colored borders), a monocle mode that
makes the focused window fill the screen, and vim-style directional keybindings.

## Focus Borders

A transparent overlay window rendered with `UpdateLayeredWindow` and per-pixel
alpha. The border sits on top of the focused window and changes color based
on the tiling state.

### Config

```toml
[borders]
width = 4
focused = "#00b4d8"    # light blue
monocle = "#2d6a4f"    # green
```

## Monocle Mode

Toggled with Alt+T. The focused window expands to fill the monitor work area
(with gap). Other windows remain in the workspace but are not repositioned
until monocle is turned off. The border changes to the monocle color.

## Vim Motions

Default keybindings use Alt + H/J/K/L for directional navigation:

| Shortcut        | Action             |
|-----------------|--------------------|
| Alt + J         | FocusNext          |
| Alt + K         | FocusPrev          |
| Alt + H         | FocusMonitorPrev   |
| Alt + L         | FocusMonitorNext   |
| Alt + Shift + J | SwapNext           |
| Alt + Shift + K | SwapPrev           |
| Alt + Shift + H | MoveToMonitorPrev  |
| Alt + Shift + L | MoveToMonitorNext  |
| Alt + Shift + R | Retile             |
| Alt + T         | ToggleMonocle      |

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      action.rs             # Added ToggleMonocle variant
      config/
        mod.rs              # Added BorderConfig
        keybinding.rs       # Vim-style defaults
  mosaico-windows/
    src/
      border.rs             # NEW: Color, Border overlay with UpdateLayeredWindow
      tiling.rs             # Monocle mode, border integration
      lib.rs                # Added border module
  mosaico/
    src/
      main.rs               # Added ToggleMonocle CLI subcommand
```

## Tasks

- [x] Add ToggleMonocle to Action enum
- [x] Add BorderConfig to config (width, focused color, monocle color)
- [x] Implement Border overlay window with UpdateLayeredWindow
- [x] Add monocle mode to TilingManager
- [x] Integrate border updates on focus change and monocle toggle
- [x] Reorganize default keybindings for vim motions (Alt+HJKL)
- [x] Add ToggleMonocle CLI subcommand
- [x] Build, lint, test
- [x] Commit

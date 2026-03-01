# Phase 25: Interactive Window Resizing

**Status:** Pending

**Goal:** Allow users to resize individual BSP splits with keyboard
shortcuts, adjusting the ratio of any split node rather than only the
global ratio.

## Overview

Currently, Mosaico uses a single global `ratio` (default 0.5) for every
BSP split. Users cannot adjust individual splits — if they want more
space for a browser and less for a terminal, they must change the global
ratio which affects all splits equally.

This phase adds directional resize actions that adjust the split ratio
of the node containing the focused window. The user presses a shortcut
to grow or shrink the focused window in a given direction.

## Reference Design

```
# Before resize (50/50 split):
┌──────────┬──────────┐
│          │          │
│  Browser │ Terminal │
│          │          │
└──────────┴──────────┘

# After Alt+Ctrl+L (grow right):
┌──────────────┬───────┐
│              │       │
│  Browser     │ Term  │
│              │       │
└──────────────┴───────┘

# After Alt+Ctrl+J (grow down) on a vertical split:
┌──────────────┬───────┐
│              │       │
│  Browser     │ Term  │
│              ├───────┤
│              │ Small │
└──────────────┴───────┘
```

## Configuration

### Default Keybindings

```toml
# Resize: Alt + Ctrl + H/J/K/L
[[keybinding]]
action = "resize-left"
key = "H"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-down"
key = "J"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-up"
key = "K"
modifiers = ["alt", "ctrl"]

[[keybinding]]
action = "resize-right"
key = "L"
modifiers = ["alt", "ctrl"]
```

`Alt+Ctrl+H/J/K/L` mirrors the existing `Alt+H/J/K/L` (focus) and
`Alt+Shift+H/J/K/L` (move) patterns.

### Config option

```toml
[layout]
gap = 8
ratio = 0.5
# Amount each resize step adjusts the split ratio (0.0-1.0).
resize_delta = 0.05
```

Each keypress shifts the split ratio by `resize_delta` (default 5%).

## Architecture

### Per-Node Ratios

The current `BspLayout` uses a single global ratio. To support per-split
resizing, each split node needs its own ratio. Since BSP is computed
recursively from the window list order, the tree structure is implicit —
there's no persistent tree object.

**Approach**: Store per-split ratio overrides in a map indexed by split
position. The BSP algorithm walks the window list recursively; at each
split point, it checks the override map for a custom ratio, falling back
to the global ratio.

#### Split Key

Each BSP split is uniquely identified by the range of window indices it
covers. For a workspace with windows `[A, B, C, D]`:

```
Split [0..4] -> separates [A] from [B, C, D]   (depth 0)
Split [1..4] -> separates [B] from [C, D]       (depth 1)
Split [2..4] -> separates [C] from [D]          (depth 2)
```

The split key is `(start_index, end_index)` of the window range. This
is stable as long as windows aren't added/removed. When windows change,
overrides for stale ranges are pruned.

#### Data Structure

In `Workspace` (mosaico-core):

```rust
pub struct Workspace {
    handles: Vec<usize>,
    /// Per-split ratio overrides. Key is (start, end) of the window
    /// range at the split point. Absent entries use the global ratio.
    split_ratios: HashMap<(usize, usize), f64>,
}
```

In `BspLayout::split()`, replace `self.ratio` with:

```rust
let ratio = split_ratios
    .get(&(start, start + handles.len()))
    .copied()
    .unwrap_or(self.ratio);
```

### Resize Logic

When the user presses `resize-right` on the focused window:

1. Find which split node the focused window belongs to
2. Determine if `right` maps to growing or shrinking at that split:
   - If the focused window is on the **left** side of a horizontal
     split, growing right means increasing the ratio
   - If the focused window is on the **right** side, growing right
     means decreasing the ratio (the left side shrinks)
3. Adjust the ratio by `+/- resize_delta`, clamped to `0.1..0.9`
4. Store in `split_ratios`
5. Re-tile the workspace

#### Direction Mapping

For a horizontal split (left | right):
- Window on left + `resize-right` -> ratio += delta (grow)
- Window on left + `resize-left` -> ratio -= delta (shrink)
- Window on right + `resize-right` -> ratio += delta (shrink focused)
- Window on right + `resize-left` -> ratio -= delta (grow focused)

For a vertical split (top | bottom):
- Window on top + `resize-down` -> ratio += delta (grow)
- Window on top + `resize-up` -> ratio -= delta (shrink)
- Window on bottom + `resize-down` -> ratio += delta (shrink focused)
- Window on bottom + `resize-up` -> ratio -= delta (grow focused)

The key insight: the direction indicates which edge of the focused
window moves. `resize-right` moves the right edge of the focused window
rightward (or the nearest split boundary in that direction).

### New Actions

```rust
pub enum Action {
    // ... existing ...
    Resize(Direction),       // resize-left, resize-right, resize-up, resize-down
}
```

### Reset Ratios

Add `retile` (Alt+Shift+R) behavior: when the user retiles, all
per-split ratio overrides are cleared, restoring the global ratio.
This gives users a quick "reset to default" escape hatch.

### Changes to `compute_layout`

`Workspace::compute_layout()` passes `&self.split_ratios` to the layout
engine so it can look up per-node ratios during recursion.

## Modified Files

```
crates/
  mosaico-core/
    src/
      action.rs              # Add Resize(Direction)
      layout.rs              # Accept split_ratios in split()
      workspace.rs           # Add split_ratios field, adjust_ratio()
      config/
        mod.rs               # Add resize_delta to LayoutConfig
        keybinding.rs        # Add default resize keybindings
        template.rs          # Add resize bindings to template
  mosaico-windows/
    src/
      tiling/
        mod.rs               # Handle Resize action
        layout.rs            # Pass split_ratios to compute_layout
  mosaico/
    src/
      main.rs                # Add Resize CLI subcommand
      commands/
        action.rs            # Route Resize action
```

## Edge Cases

1. **Single window**: No split to resize. `resize-*` is a no-op.
2. **Two windows**: One split. All four resize directions affect it.
3. **Monocle mode**: Resize is blocked (like move).
4. **Window added/removed**: Prune `split_ratios` entries whose key
   range no longer matches the current window count.
5. **Workspace switch**: Each workspace has its own `split_ratios`.
   Switching workspaces naturally uses the correct overrides.
6. **Config reload**: `resize_delta` updates immediately. Existing
   overrides are preserved (not reset).

## Tasks

- [ ] Add `resize_delta: f64` to `LayoutConfig` (default 0.05)
- [ ] Add `resize_delta` to `generate_config()` template
- [ ] Add `Resize(Direction)` variant to `Action` enum
- [ ] Update `Action::from_str` / `Display` for resize actions
- [ ] Add default keybindings: Alt+Ctrl+H/J/K/L -> resize directions
- [ ] Add `split_ratios: HashMap<(usize, usize), f64>` to `Workspace`
- [ ] Update `BspLayout::split()` to accept and use split ratio overrides
- [ ] Update `Workspace::compute_layout()` to pass split_ratios
- [ ] Implement `adjust_split_ratio()` on `Workspace`:
      find the split containing the focused window, adjust ratio
- [ ] Implement resize handler in `TilingManager::handle_action()`
- [ ] Clear `split_ratios` on retile (reset to global ratio)
- [ ] Prune stale `split_ratios` when windows are added/removed
- [ ] Add `Resize` subcommand to CLI (action resize left/right/up/down)
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: resize horizontal split left/right
- [ ] Manual test: resize vertical split up/down
- [ ] Manual test: resize with 3+ windows (nested splits)
- [ ] Manual test: retile resets all ratios
- [ ] Manual test: resize clamped at 0.1/0.9
- [ ] Update documentation (`docs/configuration.md`, `docs/tiling-layout.md`)
- [ ] Update `.plans/plan.md`

# Phase 31: Unfocused Window Borders

**Status:** Pending

**Goal:** Add subtle borders around non-focused tiled windows so users can
visually distinguish window boundaries, especially with small gaps.

## Overview

Currently mosaico uses a single `Border` overlay that follows the focused
window. This phase adds a border to every tiled window on the active
workspace, with a configurable unfocused color (default: a subtle gray).

## Reference Design

```
┌─ focused (blue) ──────┬─ unfocused (gray) ────┐
│                        │                        │
│       Terminal         │       Browser          │
│                        │                        │
├─ unfocused (gray) ─────┼─ unfocused (gray) ────┤
│                        │                        │
│       Editor           │       Docs             │
│                        │                        │
└────────────────────────┴────────────────────────┘
```

## Configuration

```toml
[borders]
width = 4
corner_style = "small"
focused = "blue"
monocle = "green"
unfocused = "#313244"    # NEW — Catppuccin surface0, subtle dark gray
                          # Set to "" to disable unfocused borders
```

Theme-aware: `unfocused` resolves named colors from the active theme just
like `focused` and `monocle`.

## Architecture

### Current: Single Border

```
TilingManager {
    border: Option<Border>,   // one overlay, moves on focus change
}
```

### New: Border Pool

```
TilingManager {
    borders: HashMap<usize, Border>,  // hwnd → border overlay
    focused_border: Option<Border>,   // separate focused border (on top)
}
```

Alternatively, keep a `Vec<Border>` pool and reuse them as windows are
added/removed, to avoid creating/destroying Win32 windows frequently.

### Key Changes

1. **Border pool management** — create/destroy borders as windows are
   added/removed from workspaces.

2. **Update all borders on layout change** — `apply_layout_on()` must
   update every border position, not just the focused one.

3. **Focus change** — when focus moves, the old focused window gets the
   unfocused color and the new one gets the focused color. No need to
   move borders — just recolor them.

4. **Workspace switch** — hide all borders on the old workspace, show
   them on the new workspace.

5. **Monocle mode** — show only the focused border in monocle color,
   hide unfocused borders.

6. **Performance** — each border is a layered Win32 window with
   `UpdateLayeredWindow`. With 8–10 windows this means 8–10 overlay
   windows. Should be fine for typical usage but worth profiling.

### Disabled by Default?

Consider whether unfocused borders should be enabled by default or
opt-in. The empty string `unfocused = ""` disables them, preserving
current behavior for users who prefer minimal borders.

## Modified Files

```
crates/
  mosaico-core/
    src/
      config/types.rs        # Add unfocused field to BorderConfig
      config/theme.rs        # Resolve unfocused color from theme
      config/template_config.rs  # Add unfocused to config template
  mosaico-windows/
    src/
      border.rs              # Border pool or per-window border management
      tiling/
        mod.rs               # Replace Option<Border> with border pool
        focus.rs             # Update all borders on focus change
        layout.rs            # Update all border positions after layout
        lifecycle.rs         # Create/destroy borders on window add/remove
        workspace.rs         # Hide/show borders on workspace switch
```

## Edge Cases

1. **Many windows** — 10+ borders may impact rendering. Pool and reuse.
2. **Workspace switch** — must hide all borders on old ws, show on new.
3. **Monitor change** — borders must follow windows across monitors.
4. **Monocle** — only focused border visible in monocle color.
5. **Config reload** — update all border colors without recreating them.
6. **Disabled** — empty `unfocused` string means no unfocused borders.

## Tasks

- [ ] Add `unfocused: String` to `BorderConfig` (default: "")
- [ ] Resolve unfocused color from theme in `validate()`
- [ ] Add `unfocused` to config template
- [ ] Replace `Option<Border>` with border pool in `TilingManager`
- [ ] Create borders for all tiled windows on active workspace
- [ ] Update all border positions in `apply_layout_on()`
- [ ] Recolor borders on focus change (focused/unfocused swap)
- [ ] Hide/show borders on workspace switch
- [ ] Handle monocle mode (hide unfocused borders)
- [ ] Handle window add/remove (create/destroy border)
- [ ] Add unit tests
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings`
- [ ] Run `cargo test --lib`
- [ ] Manual test: verify borders appear on all windows
- [ ] Manual test: verify focus change recolors correctly
- [ ] Manual test: verify workspace switch hides/shows borders
- [ ] Manual test: verify monocle hides unfocused borders
- [ ] Update `.plans/plan.md`

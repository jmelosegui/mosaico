# Phase 27: Additional Layouts & Layout Cycling

**Status:** Pending

**Goal:** Add Columns, Rows, and VerticalStack layouts alongside the
existing BSP layout, and allow users to cycle through layouts with a
keyboard shortcut.

## Overview

Mosaico currently only offers BSP (Binary Space Partitioning). While
BSP is versatile, users often want simpler, more predictable layouts:

- **Columns**: Equal-width vertical columns, one per window
- **Rows**: Equal-height horizontal rows, one per window
- **VerticalStack**: Master pane on the left, vertical stack on the right

This phase adds these three layouts, a layout cycling mechanism, and
per-monitor layout tracking so each monitor can use a different layout.

## Reference Design

```
# Columns layout (3 windows):
┌───────┬───────┬───────┐
│       │       │       │
│   A   │   B   │   C   │
│       │       │       │
└───────┴───────┴───────┘

# Rows layout (3 windows):
┌───────────────────────┐
│           A           │
├───────────────────────┤
│           B           │
├───────────────────────┤
│           C           │
└───────────────────────┘

# VerticalStack layout (3 windows, ratio = 0.5):
┌───────────┬───────────┐
│           │     B     │
│     A     ├───────────┤
│ (master)  │     C     │
└───────────┴───────────┘

# Cycling: Alt+N -> BSP -> Columns -> Rows -> VerticalStack -> BSP -> ...
```

## Configuration

### Default Keybinding

```toml
# Cycle layout: Alt + N
[[keybinding]]
action = "cycle-layout"
key = "N"
modifiers = ["alt"]
```

`Alt+N` (N for next layout) is unbound and easy to reach.

### Config option

```toml
[layout]
gap = 8
ratio = 0.5
# Default layout: "bsp", "columns", "rows", or "vertical-stack".
default = "bsp"
```

The `default` setting controls which layout new monitors start with.
The `ratio` setting applies to BSP and VerticalStack (master/stack
split). Columns and Rows ignore it (equal distribution).

## Architecture

### Layout Enum

Replace the concrete `BspLayout` stored in `TilingManager` with a
layout enum:

```rust
/// Available tiling layout algorithms.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayoutKind {
    /// Binary Space Partitioning — recursive subdivision.
    #[default]
    Bsp,
    /// Equal-width vertical columns.
    Columns,
    /// Equal-height horizontal rows.
    Rows,
    /// Master pane on the left, vertical stack on the right.
    #[serde(rename = "vertical-stack")]
    VerticalStack,
}
```

### Layout Trait (existing)

The existing `Layout` trait in `mosaico-core/src/layout.rs`:

```rust
pub trait Layout {
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)>;
}
```

Each new layout implements this trait.

### New Layout Implementations

#### Columns

```rust
pub struct ColumnsLayout { pub gap: i32 }

impl Layout for ColumnsLayout {
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)> {
        // Divide work_area width equally among handles
        // Each window gets width = (work_area.width - gaps) / n
        // All windows have full height
    }
}
```

#### Rows

```rust
pub struct RowsLayout { pub gap: i32 }

impl Layout for RowsLayout {
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)> {
        // Divide work_area height equally among handles
        // Each window gets height = (work_area.height - gaps) / n
        // All windows have full width
    }
}
```

#### VerticalStack

```rust
pub struct VerticalStackLayout { pub gap: i32, pub ratio: f64 }

impl Layout for VerticalStackLayout {
    fn apply(&self, handles: &[usize], work_area: &Rect) -> Vec<(usize, Rect)> {
        // First window (master) gets left portion: width * ratio
        // Remaining windows split the right portion equally (vertical stack)
        // Single window: fills entire work area
    }
}
```

### Per-Monitor Layout

Currently `TilingManager` stores a single `layout: BspLayout`. Change
to per-monitor layout tracking:

```rust
struct MonitorState {
    // ... existing fields ...
    layout_kind: LayoutKind,   // per-monitor active layout
}
```

The `TilingManager` stores shared layout parameters (gap, ratio) and
each monitor tracks which layout kind is active.

### Layout Cycling

The cycle order is: `BSP -> Columns -> Rows -> VerticalStack -> BSP -> ...`

```rust
impl LayoutKind {
    fn next(self) -> Self {
        match self {
            Self::Bsp => Self::Columns,
            Self::Columns => Self::Rows,
            Self::Rows => Self::VerticalStack,
            Self::VerticalStack => Self::Bsp,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Bsp => "BSP",
            Self::Columns => "Columns",
            Self::Rows => "Rows",
            Self::VerticalStack => "VStack",
        }
    }
}
```

### Applying the Active Layout

`apply_layout_on()` checks the monitor's `layout_kind` and dispatches:

```rust
fn apply_layout_on(&mut self, mon_idx: usize) {
    let kind = self.monitors[mon_idx].layout_kind;
    let gap = self.layout_gap;
    let ratio = self.layout_ratio;

    let layout: Box<dyn Layout> = match kind {
        LayoutKind::Bsp => Box::new(BspLayout { gap, ratio }),
        LayoutKind::Columns => Box::new(ColumnsLayout { gap }),
        LayoutKind::Rows => Box::new(RowsLayout { gap }),
        LayoutKind::VerticalStack => Box::new(VerticalStackLayout { gap, ratio }),
    };

    // ... compute positions and set window rects ...
}
```

Alternatively, avoid boxing by using an enum dispatch inline, which is
more efficient and idiomatic for a fixed set of layouts.

### Bar Display

`bar_states()` currently hardcodes "BSP". Change to read from
`MonitorState::layout_kind.name()`:

```rust
layout_name: self.monitors[mon_idx].layout_kind.name().into(),
```

The layout widget already displays `layout_name`, so this works
automatically. In monocle mode, show "VStack | M" etc.

### New Action

```rust
pub enum Action {
    // ... existing ...
    CycleLayout,    // cycle-layout
}
```

### Resize Interaction (Phase 25)

Per-split ratio overrides from Phase 25 only apply to BSP. When cycling
to a different layout, the overrides are preserved but not used.
Cycling back to BSP restores them. This is correct behavior — ratios
are BSP-specific.

### Spatial Navigation

`find_same_monitor_neighbor()` uses layout positions for spatial
navigation. Since all layouts produce `Vec<(usize, Rect)>`, spatial
navigation works automatically with any layout — no changes needed.

## Modified Files

```
crates/
  mosaico-core/
    src/
      action.rs              # Add CycleLayout
      layout.rs              # Add ColumnsLayout, RowsLayout, VerticalStackLayout,
                             # LayoutKind enum
      config/
        mod.rs               # Add default layout to LayoutConfig
        keybinding.rs        # Add Alt+N default keybinding
        template.rs          # Add cycle-layout, default layout option
  mosaico-windows/
    src/
      tiling/
        mod.rs               # Per-monitor layout_kind, handle CycleLayout
        layout.rs            # Dispatch to correct layout impl
      bar/
        widgets/
          layout.rs          # Use dynamic layout name
  mosaico/
    src/
      main.rs                # Add CycleLayout CLI subcommand
```

## Edge Cases

1. **Single window**: All layouts place a single window identically
   (fills the work area minus gap). Cycling has no visible effect.

2. **Two windows**: Columns and VerticalStack look identical (two
   vertical panes). Rows shows two horizontal panes. BSP starts with
   a horizontal split.

3. **VerticalStack + resize (Phase 25)**: The master/stack split ratio
   could be adjustable. VerticalStack uses the global `ratio` for the
   master split. Phase 25 per-split overrides don't apply here — future
   enhancement could add master ratio adjustment.

4. **Monocle + cycle**: Layout cycling while in monocle mode changes
   the stored layout kind but the visual stays monocle. Exiting monocle
   reveals the new layout.

5. **Config reload**: If the user changes `default = "columns"` in
   config, this sets the default for new monitors but doesn't change
   the active layout on existing monitors (they keep their cycled
   selection). To reset, the user can cycle back or restart.

6. **Many windows in Columns**: With 10+ windows, each column becomes
   very narrow. This is expected — the user should cycle to BSP or
   VerticalStack for many windows.

## Tasks

- [ ] Add `LayoutKind` enum to `mosaico-core/src/layout.rs`
- [ ] Implement `ColumnsLayout` with `Layout` trait
- [ ] Implement `RowsLayout` with `Layout` trait
- [ ] Implement `VerticalStackLayout` with `Layout` trait
- [ ] Add `next()` and `name()` methods to `LayoutKind`
- [ ] Add `default: LayoutKind` to `LayoutConfig` (default `Bsp`)
- [ ] Add `default = "bsp"` to `generate_config()` template
- [ ] Add `layout_kind: LayoutKind` to `MonitorState`
- [ ] Update `apply_layout_on()` to dispatch based on `layout_kind`
- [ ] Add `CycleLayout` variant to `Action` enum
- [ ] Update `Action::from_str` / `Display` for `cycle-layout`
- [ ] Add default keybinding: Alt+N -> CycleLayout
- [ ] Add `cycle-layout` to keybindings template comments
- [ ] Implement `cycle_layout()` in `TilingManager`:
      advance focused monitor's `layout_kind`, retile
- [ ] Update `bar_states()` to use `layout_kind.name()`
- [ ] Initialize `MonitorState::layout_kind` from config default
- [ ] Add `CycleLayout` subcommand to CLI
- [ ] Add unit tests for each layout (1, 2, 3, 5 windows)
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: cycle through all 4 layouts, verify correct tiling
- [ ] Manual test: Columns with 1, 3, and 6 windows
- [ ] Manual test: Rows with 1, 3, and 6 windows
- [ ] Manual test: VerticalStack with 1, 2, and 4 windows
- [ ] Manual test: cycle layout while in monocle mode
- [ ] Manual test: verify bar displays correct layout name
- [ ] Manual test: spatial navigation works with all layouts
- [ ] Update documentation (`docs/tiling-layout.md`, `docs/configuration.md`,
      `docs/keyboard-bindings.md`)
- [ ] Update `.plans/plan.md`

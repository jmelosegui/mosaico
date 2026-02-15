# Phase 7: Tiling Layouts

**Status:** Complete

**Goal:** Implement a layout algorithm that automatically arranges windows in a
tiled pattern when they are opened, closed, minimized, or restored.

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      layout.rs               # Layout trait + BSP layout algorithm
      workspace.rs             # Managed window list
  mosaico-windows/
    src/
      monitor.rs               # Work area queries (MonitorFromWindow)
      tiling.rs                # TilingManager — event handling + layout
      daemon.rs                # Updated to use TilingManager
```

## Tasks

- [x] Define `Layout` trait in `mosaico-core`
- [x] Implement BSP (Binary Space Partitioning) layout
- [x] Create `Workspace` to track managed window ordering
- [x] Add monitor work area query in `mosaico-windows`
- [x] Create `TilingManager` to process events and apply layout
- [x] Integrate into daemon — auto-tile on window create/destroy/minimize/restore
- [x] Add unit tests for layout and workspace
- [x] Commit

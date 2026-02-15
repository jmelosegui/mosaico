# Phase 6: Window Positioning

**Status:** Complete

**Goal:** Give the daemon the ability to move and resize windows programmatically.
This is the first step toward actual tiling behavior.

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      window.rs               # Add move/resize methods to Window trait
  mosaico-windows/
    src/
      window.rs               # Implement move/resize using SetWindowPos
```

## Tasks

- [x] Add `set_rect(&self, rect: Rect)` to the `Window` trait in `mosaico-core`
- [x] Implement `set_rect` in `mosaico-windows` using `SetWindowPos`
- [x] Handle DPI awareness (`SetProcessDpiAwarenessContext`) so positions are
      accurate on high-DPI monitors
- [x] Handle `DWMWA_EXTENDED_FRAME_BOUNDS` for accurate window bounds (windows
      have invisible borders that affect sizing)
- [x] Add a `mosaico debug move` command for manual testing (e.g.
      `mosaico debug move --hwnd 0x1234 --x 0 --y 0 --w 960 --h 1080`)
- [x] React to window events from Phase 5 — when a new window appears, position
      it according to a simple rule (moved to Phase 7)
- [x] Commit

## Win32 APIs used

| Function | Purpose |
|----------|---------|
| `SetWindowPos` | Move and resize a window, set Z-order |
| `MoveWindow` | Simpler move/resize (used for animations) |
| `DwmGetWindowAttribute` | Get accurate window bounds (DWMWA_EXTENDED_FRAME_BOUNDS) |
| `SetProcessDpiAwarenessContext` | Declare DPI awareness for accurate coordinates |
| `GetDpiForMonitor` | Get per-monitor DPI scaling factor |
| `ShowWindow` | Restore a window before positioning |

## Rust learning notes

- **`&self` vs `&mut self`** — `set_rect` takes `&self` (immutable reference) even
  though it modifies the window, because the modification happens on the OS side
  via the HWND handle. The Rust struct itself doesn't change. This is called
  "interior mutability via FFI" — the mutation is external to Rust's ownership
  model.
- **DPI scaling** — on high-DPI monitors, pixel coordinates don't map 1:1 to
  physical pixels. Windows scales them by a factor (e.g. 150%). The WM must be
  DPI-aware to position windows correctly.

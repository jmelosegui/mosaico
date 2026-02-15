# Phase 3b: Window Enumeration (`mosaico list`)

**Status:** Complete

**Goal:** Enumerate visible top-level windows on Windows and display their info
via a new `mosaico list` subcommand.

## Target structure (new/modified files only)

```
crates/
  mosaico/
    src/
      commands/
        list.rs               # new — calls platform code, formats output
  mosaico-core/
    src/
      window.rs               # Window trait with title(), class(), rect(), etc.
      rect.rs                 # Rect struct (x, y, width, height)
  mosaico-windows/
    src/
      window.rs               # Window struct implementing core::Window
      enumerate.rs            # EnumWindows wrapper
      styles.rs               # Window style constants and helpers
      error.rs                # Win32 error helpers
```

## Tasks

- [x] Add the `windows` crate to `mosaico-windows` with required Win32 features
- [x] Define the `Window` trait in `mosaico-core` (title, class, rect, is_visible)
- [x] Implement `Window` struct in `mosaico-windows` wrapping an `HWND`
- [x] Implement `enumerate_windows()` in `mosaico-windows/src/enumerate.rs`
- [x] Filter to visible, non-minimized, standard application windows
- [x] Add `list` subcommand to the CLI under `debug` subgroup
- [x] Add integration test for `mosaico debug list`
- [x] Commit

## Win32 APIs used

| Function | Purpose |
|----------|---------|
| `EnumWindows` | Iterate over all top-level windows |
| `GetWindowTextW` | Get window title |
| `RealGetWindowClassW` | Get window class name |
| `GetWindowRect` | Get window position and size |
| `IsWindowVisible` | Check if window is visible |
| `IsIconic` | Check if window is minimized |
| `GetWindowLongPtrW` | Read window styles (WS_CAPTION, etc.) |
| `GetWindowThreadProcessId` | Get the owning process ID |

## Rust learning notes

- `unsafe` blocks — required for calling Win32 FFI functions. The compiler can't
  verify memory safety across the FFI boundary, so you must explicitly opt in.
  Keep `unsafe` blocks as small as possible and wrap them in safe functions.
- `trait` — like a C# `interface`. `mosaico-core` defines `trait Window`,
  `mosaico-windows` implements it for its own `Window` struct.
- `cfg(target_os)` — conditional compilation. Code behind
  `#[cfg(target_os = "windows")]` is invisible to the compiler on other platforms.
- `windows` crate — Microsoft's official Rust bindings for Win32. You enable only
  the API surface you need via feature flags in `Cargo.toml`.

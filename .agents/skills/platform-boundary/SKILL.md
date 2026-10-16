---
name: platform-boundary
description: Keep mosaico-core OS-agnostic. Use when adding or changing code that talks to the OS, when reviewing a diff that touches mosaico-core, mosaico-windows or the CLI commands, or when auditing the crates for missing core contracts.
---

# Platform boundary

## The layering

| Layer | Owns | Must not |
| --- | --- | --- |
| `mosaico-core` | Traits for every OS capability, their contracts, data types, and all decision logic: tiling, workspaces, navigation, monitor matching, config parsing and reload rules, bar content, version checks | depend on `windows`, use `unsafe` or `cfg(target_os)`, read OS-specific env vars, mention OS APIs |
| `mosaico-windows` | Thin implementations of core traits: FFI calls, converting native handles and events into core types, Win32 quirks documented next to the code that handles them | own policy, hold state that core needs to reason about, expose native types to callers |
| `mosaico` (CLI) | Argument parsing and output | call `mosaico_windows::*` from command handlers; pick the platform in one place (a `platform` module behind `cfg`) |

A good test: could a `mosaico-linux` crate make the feature work by implementing traits only? If it would have
to copy a function out of `mosaico-windows`, that function is in the wrong crate.

## When you add or change OS functionality

1. Find the trait in core that covers the capability. If none does, add it (or a method on an existing trait) in
   core before writing platform code.
2. Write the contract as doc comments on the trait, in OS-neutral terms:
   - what the method guarantees and when it may fail
   - invariants (stability across sessions, ordering, uniqueness)
   - what a platform should do when it cannot support the feature (return `None`, report unsupported, fall back)
   - side effects callers depend on (for example, whether hiding a window emits a window event)
3. Put the decision logic in core and unit test it there against a fake implementation of the trait.
4. Implement the trait in `mosaico-windows`. Win32 details, API names and workarounds go in that file's docs.
5. Make callers (tiler, daemon loop, CLI) use the trait. Do not add an inherent `pub fn` on a platform type
   that callers use instead.
6. Identify things with opaque core types, never with raw `usize` handles or sentinel values like `0`.
7. Name config options and enum variants after behaviour the user sees ("keeps taskbar entry"), not the API that
   implements it ("DWM cloak").

The contract docs on a trait should read like this:

```rust
/// Identity of a physical display that survives reconnects.
///
/// # Contract for platform implementations
///
/// - The key must stay the same across unplugging, sleep, reboots and
///   resolution changes.
/// - It must never be built from per-session handles or desktop position.
/// - Leave a field `None` rather than invent a value.
```

## Red flags

Any of these in a diff means a contract is missing or being bypassed:

- `mosaico_windows::` used in `crates/mosaico/src/commands/` or `main.rs`
- `windows::Win32` imported in tiling, daemon loop, bar widget logic or other decision code
- `Window::from_raw(`, `.hwnd()` or a native handle passed around outside the module that wraps it
- an inherent `pub fn` on a platform type (`Window`, `Border`, ...) called from the tiler while
  the core trait lacks an equivalent
- a core trait that is implemented but never used through the trait
- `usize` window or monitor ids named `hwnd`/`hmonitor`, or `0` meaning "no window"
- `std::env::var("USERPROFILE" | "LOCALAPPDATA" | "APPDATA" | "HOME")` or path separators hardcoded in core
- core docs or types mentioning `HWND`, `HMONITOR`, DWM, `ShowWindow`, `SetWindowPos`, `EVENT_OBJECT_*`,
  virtual key codes, registry, named pipes, `.exe`
- pure logic (parsing, formatting, mtime comparison, version comparison, validation) in a platform file next to
  FFI
- shared code hardcoding release artifacts (`mosaico.exe`, `mosaico-windows-amd64.zip`)
- platform functions with their own error style (`Result<_, String>`) instead of the core error type

## Audit recipe

Run these from the repo root when auditing or reviewing:

```sh
# CLI reaching into the platform crate
grep -rn "mosaico_windows::" crates/mosaico/src

# Win32 inside decision logic
grep -rln "windows::Win32" crates/mosaico-windows/src/tiling crates/mosaico-windows/src/daemon_loop*.rs

# OS concepts leaking into core
grep -rniE "hwnd|hmonitor|dwm|win32|registry|USERPROFILE|LOCALAPPDATA|\.exe" crates/mosaico-core/src

# Inherent methods on platform types vs. what the core trait offers
grep -n "pub fn" crates/mosaico-windows/src/window.rs
grep -n "fn " crates/mosaico-core/src/window.rs
```

For each hit, decide whether it is a missing contract, a bypassed contract or logic in the wrong crate.

## Known gaps

The open gaps are tracked under the umbrella issue #41 (one issue each, #42 to #55). Check them before filing a
new one, and link new boundary issues to #41. When your change closes part of a gap, say so in the commit.

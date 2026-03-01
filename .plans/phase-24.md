# Phase 24: Single-Instance Daemon Guard

**Status:** Pending

**Goal:** Ensure only one Mosaico daemon instance can run at a time.

## Overview

Starting the daemon twice can lead to conflicting window management,
duplicated event hooks, and hard-to-debug behaviour. This phase adds a
single-instance guard so the daemon exits cleanly when another instance
is already running.

## Strategy

Use a named, system-wide mutex to detect an existing instance. The
first daemon creates the mutex and keeps it alive for the lifetime of
the process. A second daemon sees the mutex already exists and exits
with a clear error message.

## Architecture

### Windows Implementation

Add a new helper in `mosaico-windows` responsible for creating and
holding the mutex handle:

```rust
pub struct InstanceGuard {
    handle: windows::Win32::Foundation::HANDLE,
}

impl InstanceGuard {
    pub fn acquire(name: &str) -> Result<Self, InstanceError> {
        // CreateMutexW with a fixed, namespaced identifier
        // If ERROR_ALREADY_EXISTS, return InstanceError::AlreadyRunning
    }
}
```

Use a stable name such as `Global\\MosaicoDaemon` so it applies across
sessions. The guard is kept alive by storing it in the daemon runtime
state. Dropping it on shutdown releases the mutex automatically.

### Error Type

Introduce a small error enum to provide a user-facing message:

```rust
pub enum InstanceError {
    AlreadyRunning,
    Os(windows::core::Error),
}
```

### CLI Flow

During daemon startup (the `daemon` subcommand), attempt to acquire the
guard before initializing IPC or window hooks. On `AlreadyRunning`,
print a concise error and exit with a non-zero status.

## User Experience

- First daemon starts normally.
- Second daemon exits with:
  "mosaico daemon is already running"

No auto-shutdown or takeover logic is added in this phase; the user can
use `mosaico stop` to stop the running daemon.

## Modified Files

```
crates/
  mosaico-windows/
    src/
      instance_guard.rs       # New mutex guard implementation
      lib.rs                  # Re-export InstanceGuard
  mosaico/
    src/
      daemon.rs               # Acquire guard before startup
```

## Edge Cases

1. **Crash recovery**: If the daemon crashes, the OS releases the mutex
   automatically. Next start succeeds.

2. **Service/Session boundary**: Using the `Global\\` prefix ensures
   the mutex is visible across user sessions (matches auto-start).

3. **Permission errors**: If `CreateMutexW` fails for reasons other than
   `ERROR_ALREADY_EXISTS`, surface the OS error message.

## Tasks

- [ ] Add `instance_guard.rs` to `mosaico-windows` with `InstanceGuard`
- [ ] Use `CreateMutexW` and check `ERROR_ALREADY_EXISTS`
- [ ] Define `InstanceError` with user-friendly messaging
- [ ] Acquire guard in daemon startup before IPC
- [ ] Store guard for daemon lifetime (drop on shutdown)
- [ ] Add integration test: starting daemon twice yields error
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [ ] Run `cargo test` and fix failures
- [ ] Manual test: start daemon twice, verify second exits cleanly
- [ ] Update `.plans/plan.md`

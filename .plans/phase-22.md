# Phase 22: Self-Update Command

**Status:** Complete

**Goal:** Add a `mosaico update` command that checks for a newer release,
stops the running daemon (if any), downloads and replaces the binary,
and restarts the daemon automatically.

## Overview

Currently, updating Mosaico requires the user to manually run the
PowerShell install script (`irm ... | iex`). The `mosaico start` banner
and the status bar widget already detect when an update is available, but
the user has to leave the terminal and run a separate command to install
it. This phase adds a first-class `mosaico update` CLI command that
performs the full update cycle in one step.

## Reference Design

```
$ mosaico update
Checking for updates...
Update available: v0.1.0 -> v0.2.0
Stopping daemon...
Downloading v0.2.0...
Installing to C:\Users\jose\AppData\Local\mosaico\mosaico.exe...
Updated successfully: v0.1.0 -> v0.2.0
Restarting daemon...
Mosaico is running (PID: 12345).

$ mosaico update
You are already on the latest version (v0.1.0).

$ mosaico update --force
Stopping daemon...
Downloading v0.1.0...
Installing to C:\Users\jose\AppData\Local\mosaico\mosaico.exe...
Reinstalled v0.1.0.
Restarting daemon...
Mosaico is running (PID: 12346).
```

## Architecture

### Update Flow

```
mosaico update
  |
  +-- 1. Check GitHub API for latest release tag
  |       (reuse existing version_check logic)
  |
  +-- 2. Compare with local version
  |       If same and no --force: print "already latest", exit
  |
  +-- 3. Was daemon running? Remember this (IPC check)
  |
  +-- 4. Stop daemon if running (reuse stop logic)
  |
  +-- 5. Download release zip from GitHub
  |       URL: https://github.com/jmelosegui/mosaico/releases/download/{tag}/mosaico-windows-amd64.zip
  |
  +-- 6. Extract mosaico.exe from zip to temp location
  |
  +-- 7. Replace the running binary
  |       Target: the exe that is currently running (std::env::current_exe)
  |
  +-- 8. Restart daemon if it was running (spawn new exe with "daemon" arg)
  |
  +-- 9. Print success message
```

### Self-Replacement Strategy

The tricky part is replacing a binary while it is executing. On Windows,
a running `.exe` cannot be overwritten directly — the OS locks the file.
Two approaches:

#### Option A — Rename-then-copy (Recommended)

Windows allows **renaming** a running executable (the lock prevents
writes but not renames). The flow is:

1. Rename running `mosaico.exe` → `mosaico.exe.old`
2. Copy new `mosaico.exe` from temp to the install directory
3. Delete `mosaico.exe.old` (may fail if still locked — that's OK,
   clean up on next update or startup)

This is the approach used by many self-updating Windows programs.
`std::fs::rename()` on a running `.exe` succeeds on Windows.

#### Option B — Write to temp, schedule replace on exit

More complex, requires coordination. Not needed since Option A works.

### Binary Download

The existing `http.rs` module uses WinHTTP for HTTPS GET requests, but
it currently only supports reading text responses. For downloading a zip
file, we need a binary download variant.

**Option**: Add `get_bytes(host, path, timeout_ms) -> Result<Vec<u8>>`
to `http.rs` that returns raw bytes instead of calling `String::from_utf8`.

However, GitHub release asset downloads involve a redirect from
`github.com` to `objects.githubusercontent.com`. WinHTTP follows
redirects automatically by default, so this should work transparently.

### Zip Extraction

The zip file contains a single `mosaico.exe` at the root. For minimal
dependencies, we can use the `zip` crate (already available in the Rust
ecosystem) or implement minimal zip extraction for a single file.

**Recommendation**: Use the `zip` crate. It's well-maintained, adds
minimal compile time, and handles edge cases (compression methods,
CRC validation). The alternative (manual zip parsing) is fragile and
not worth the complexity for saving one dependency.

## New Files

```
crates/
  mosaico/
    src/
      commands/
        update.rs              # Update command implementation
  mosaico-windows/
    src/
      http.rs                  # Add get_bytes() for binary downloads
```

## Modified Files

```
crates/
  mosaico/
    src/
      main.rs                  # Register Update subcommand
      commands/
        mod.rs                 # Add pub mod update
        start.rs               # Update the "Run the install script" hint
  mosaico-windows/
    Cargo.toml                 # Add zip dependency
```

## Key Types and Functions

### `mosaico-windows/src/http.rs`

Add a binary download function alongside the existing text `get()`:

```rust
/// Performs a synchronous HTTPS GET and returns the raw response body.
///
/// Unlike `get()`, this returns raw bytes without UTF-8 conversion,
/// suitable for downloading binary files (zip archives, etc.).
pub fn get_bytes(host: &str, path: &str, timeout_ms: i32) -> Result<Vec<u8>, String> {
    // Same as get() but skip String::from_utf8
}
```

### `mosaico/src/commands/update.rs`

```rust
/// Arguments for the update command.
pub struct UpdateArgs {
    /// Force reinstall even if already on the latest version.
    pub force: bool,
}

pub fn execute(args: &UpdateArgs) {
    // 1. Check for update
    // 2. Stop daemon if running
    // 3. Download and install
    // 4. Restart daemon if it was running
}
```

#### Step 1: Check for update

Reuse `version_check::check_for_update()` for the remote tag, but also
need the raw tag even when versions are equal (for `--force`). Factor
out a helper:

```rust
/// Returns the latest release tag from GitHub (e.g. "v0.2.0"),
/// or None on network error.
fn fetch_latest_tag() -> Option<String> {
    let body = mosaico_windows::http::get(GITHUB_HOST, RELEASES_PATH, TIMEOUT_MS).ok()?;
    extract_tag_name(&body)
}
```

This can be extracted from the existing `version_check.rs` module and
made `pub` so the update command can reuse it.

#### Step 2: Stop daemon

```rust
fn was_daemon_running() -> bool {
    mosaico_windows::ipc::is_daemon_running()
}

fn stop_daemon() {
    super::stop::execute();
}
```

#### Step 3: Download and extract

```rust
fn download_release(tag: &str) -> Result<Vec<u8>, String> {
    let path = format!(
        "/repos/jmelosegui/mosaico/releases/download/{tag}/mosaico-windows-amd64.zip"
    );
    mosaico_windows::http::get_bytes("github.com", &path, 30_000)
}

fn extract_exe(zip_bytes: &[u8]) -> Result<Vec<u8>, String> {
    // Use zip crate to find "mosaico.exe" entry and read its bytes
}
```

#### Step 4: Replace binary

```rust
fn install_exe(exe_bytes: &[u8]) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("cannot determine current exe path: {e}"))?;
    let backup = current_exe.with_extension("exe.old");

    // Rename running binary out of the way
    std::fs::rename(&current_exe, &backup)
        .map_err(|e| format!("cannot rename current exe: {e}"))?;

    // Write new binary
    std::fs::write(&current_exe, exe_bytes)
        .map_err(|e| {
            // Attempt rollback
            let _ = std::fs::rename(&backup, &current_exe);
            format!("cannot write new exe: {e}")
        })?;

    // Try to delete backup (may fail if process still holds a handle)
    let _ = std::fs::remove_file(&backup);

    Ok(())
}
```

#### Step 5: Restart daemon

```rust
fn restart_daemon() {
    // Re-read the exe path (it's now the new binary)
    super::start::execute();
}
```

### `main.rs`

Add the `Update` subcommand:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Update mosaico to the latest version
    Update {
        /// Force reinstall even if already on the latest version
        #[arg(long)]
        force: bool,
    },
}
```

Route in `main()`:

```rust
Commands::Update { force } => commands::update::execute(force),
```

### `start.rs`

Update the hint when an update is available:

```rust
// Before:
println!("  Run the install script to update");
// After:
println!("  Run 'mosaico update' to install it");
```

## Error Handling

| Failure | Behaviour |
|---------|-----------|
| Network error (check) | Print error, exit 1 |
| Network error (download) | Print error, exit 1. Daemon already stopped — print hint to restart manually |
| Zip extraction fails | Print error, exit 1. Same as above |
| Rename fails | Print error, exit 1. Daemon was stopped; binary is intact |
| Write new exe fails | Attempt rollback (rename `.old` back), print error, exit 1 |
| Restart fails | Print warning. User can run `mosaico start` manually |

If the daemon was stopped but the update fails partway through, the
command should print a clear message explaining the state:

```
Error: <description>
The daemon was stopped before the update failed.
Run 'mosaico start' to restart with the current version.
```

## Design Decisions

- **Self-update over external script**: The `mosaico update` command is
  more discoverable and convenient than remembering the PowerShell
  one-liner. The install script remains available as a fallback and for
  first-time installation.

- **Rename strategy**: Windows allows renaming running executables. This
  is simpler and more reliable than writing to temp and scheduling a
  replace. Used by many real-world self-updaters (rustup, cargo-binstall).

- **`--force` flag**: Allows reinstalling the same version, useful for
  recovering a corrupted binary or verifying the update mechanism.

- **Automatic restart**: If the daemon was running before the update,
  it is restarted automatically. This minimizes disruption — the user
  doesn't need to remember to run `mosaico start` after updating.

- **No `--no-restart` flag**: Keep it simple for now. If users want
  to stop and update without restarting, they can run `mosaico stop`
  then `mosaico update` (which won't restart since daemon wasn't
  running when update started).

- **Single architecture**: Currently only `mosaico-windows-amd64.zip`
  is built. When ARM64 builds are added, the command should detect the
  current architecture and download the matching zip. For now, hardcode
  `amd64`.

- **zip crate dependency**: Adds ~30 KB to compile. Worth it for
  correctness over hand-rolled zip parsing. The alternative (shelling
  out to `tar` or PowerShell `Expand-Archive`) would break the
  single-binary, no-external-dependency design.

- **Download timeout**: 30 seconds (vs 3 seconds for the version
  check). Release zips are larger and need more time.

- **Cleanup `.old` on startup**: If `mosaico.exe.old` exists next to
  the current exe at startup, silently delete it. This handles the case
  where the old binary couldn't be deleted during the update because it
  was still locked.

## Future Considerations

- **WinGet/Scoop**: When Mosaico is published to package managers, the
  update command could detect the installation method and use the
  appropriate package manager to update. The internal download approach
  serves as the universal fallback.
- **ARM64**: Detect `std::env::consts::ARCH` and download the matching
  zip filename.
- **Rollback**: If the new version crashes on startup, the `.old` file
  could be used for automatic rollback. This is out of scope for now.
- **Release notes**: Display a summary of what changed in the new
  version (fetch from GitHub release body). Nice-to-have for a future
  phase.

## Tasks

- [x] Add `get_bytes()` to `mosaico-windows/src/http.rs`
- [x] Add `zip` crate dependency to `mosaico-windows/Cargo.toml` and
      `mosaico/Cargo.toml`
- [x] Extract `fetch_latest_tag()` from `version_check.rs` as a public
      helper (reusable by both `check_for_update` and `update`)
- [x] Make `is_newer()` public in `version_check.rs` for the update command
- [x] Remove duplicate `version_check.rs` from CLI crate (rewire to library)
- [x] Create `update.rs` in `mosaico/src/commands/`:
  - [x] `execute(force: bool)` — main entry point
  - [x] Check for update, compare versions, respect `--force`
  - [x] Detect whether daemon is running
  - [x] Stop daemon if running
  - [x] Download release zip from GitHub
  - [x] Extract `mosaico.exe` from zip
  - [x] Rename current exe to `.old`, write new exe
  - [x] Rollback on write failure
  - [x] Restart daemon if it was running
  - [x] Print clear status messages at each step
- [x] Add `pub mod update` to `commands/mod.rs`
- [x] Add `Update { force: bool }` variant to `Commands` in `main.rs`
- [x] Route `Commands::Update` to `commands::update::execute()` in `main()`
- [x] Update start.rs hint: "Run 'mosaico update' to install it"
- [x] Add `.old` file cleanup at daemon startup (best-effort delete)
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --all-targets -- -D warnings` and fix warnings
- [x] Run `cargo test --lib` and fix failures
- [x] Manual test: verify "already latest" when local >= remote
- [x] Manual test: verify `--force` reinstalls same version
- [x] Manual test: verify update when daemon is not running (no restart)
- [x] Update documentation (`docs/cli.md`, `website/src/guide/cli.md`)
- [x] Update `.plans/plan.md`

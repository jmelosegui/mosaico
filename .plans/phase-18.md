# Phase 18: Auto-Start on Windows Boot

**Status:** Planned

**Goal:** Extend the `mosaico init` command to interactively ask the user
whether Mosaico should launch automatically when Windows starts, and
configure the system accordingly.

## Overview

After generating config files, `mosaico init` prompts the user with a yes/no
question. If the user opts in, Mosaico registers itself to run at Windows
startup. If the user declines, no startup entry is created (or an existing
one is removed).

A separate `mosaico autostart` subcommand allows enabling/disabling startup
registration independently of `init`, for users who want to change the
setting later.

## Reference Design

```
$ mosaico init
Created C:\Users\jose\.config\mosaico\config.toml
Created C:\Users\jose\.config\mosaico\keybindings.toml
Created C:\Users\jose\.config\mosaico\rules.toml
Created C:\Users\jose\.config\mosaico\bar.toml

Would you like Mosaico to start automatically when Windows starts? [y/N]: y
Autostart enabled.

Edit these files to customize layout, keybindings, window rules, and the status bar.
```

```
$ mosaico autostart enable
Autostart enabled.

$ mosaico autostart disable
Autostart disabled.

$ mosaico autostart status
Autostart is currently enabled.
```

## Startup Mechanisms

Windows provides several ways to register a program to run at startup.
Two viable options:

### Option A — Registry `Run` key (Recommended)

Write a string value under:
```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
```

- **Key name**: `Mosaico`
- **Value**: full path to `mosaico.exe start` (e.g.,
  `"C:\Users\jose\.cargo\bin\mosaico.exe" start`)
- No admin rights required (HKCU is per-user)
- Survives updates (path stays the same if installed via `cargo install`)
- Easy to inspect and remove via `regedit` or programmatically
- Standard approach used by most Windows applications

### Option B — Startup folder shortcut

Create a `.lnk` shortcut in:
```
%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\
```

- Simpler (file-based, no registry)
- Less reliable (user can accidentally delete the shortcut)
- Requires creating a COM `IShellLink` object for proper `.lnk` creation

**Recommendation**: Option A (Registry Run key). It's the standard, most
reliable approach and doesn't require COM interop.

## Architecture

### New Files

```
crates/
  mosaico-windows/
    src/
      autostart.rs           # Registry-based enable/disable/status
  mosaico/
    src/
      commands/
        autostart.rs         # CLI subcommand handler
```

### Modified Files

```
crates/
  mosaico/
    src/
      commands/
        init.rs              # Add interactive prompt after config generation
      main.rs                # Register Autostart subcommand
```

### Key Types and Functions

#### `mosaico-windows/src/autostart.rs`

Platform-specific startup registration using the Windows Registry:

- `enable()` -- writes the `Mosaico` value to the `Run` key with the path
  to the current executable + ` start` argument. Returns `Result<(), String>`.
- `disable()` -- removes the `Mosaico` value from the `Run` key. Returns
  `Result<(), String>`. No-op if the value doesn't exist.
- `is_enabled()` -- checks if the `Mosaico` value exists in the `Run` key.
  Returns `bool`.
- `exe_path()` -- resolves the current executable path via
  `std::env::current_exe()` and appends ` start`.

#### `mosaico/src/commands/init.rs`

After writing config files, prompt the user:

```rust
fn prompt_autostart() {
    print!("\nWould you like Mosaico to start automatically when Windows starts? [y/N]: ");
    // flush stdout, read a line from stdin
    // if "y" or "Y", call autostart::enable()
    // otherwise, do nothing (default is no)
}
```

The default is **No** (`[y/N]`) so pressing Enter skips autostart. This
respects the principle of least surprise -- users must explicitly opt in.

#### `mosaico/src/commands/autostart.rs`

Handles the `mosaico autostart` subcommand:

```rust
pub fn execute(action: AutostartAction) {
    match action {
        AutostartAction::Enable => { /* call enable(), print result */ }
        AutostartAction::Disable => { /* call disable(), print result */ }
        AutostartAction::Status => { /* call is_enabled(), print status */ }
    }
}
```

#### `main.rs`

New subcommand enum variant:

```rust
/// Manage automatic startup when Windows boots
Autostart {
    #[command(subcommand)]
    action: AutostartAction,
},
```

```rust
#[derive(Subcommand)]
enum AutostartAction {
    /// Enable autostart on Windows boot
    Enable,
    /// Disable autostart on Windows boot  
    Disable,
    /// Show current autostart status
    Status,
}
```

## Registry Details

### Key Path

```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
```

### Value

- **Name**: `Mosaico`
- **Type**: `REG_SZ`
- **Data**: `"<path_to_mosaico.exe>" start`

Example:
```
"C:\Users\jose\.cargo\bin\mosaico.exe" start
```

The path is quoted to handle spaces. The `start` argument tells Mosaico to
launch the daemon (not show help).

### Win32 API Calls

Use the `windows` crate's registry APIs:

- `RegOpenKeyExW(HKEY_CURRENT_USER, ..., KEY_SET_VALUE | KEY_QUERY_VALUE)`
- `RegSetValueExW(key, "Mosaico", REG_SZ, path_bytes)`
- `RegDeleteValueW(key, "Mosaico")` -- for disable
- `RegQueryValueExW(key, "Mosaico", ...)` -- for status check
- `RegCloseKey(key)`

All operations use `HKEY_CURRENT_USER` so no elevation is required.

## Integration Points

- **`mosaico doctor`**: should be updated to check autostart status and
  report it (informational, not an error).
- **`mosaico init`**: prompt appears only on first run (when config files
  are being created). If all config files already exist, the prompt is
  skipped since the user likely already configured this.
- **Executable path**: `std::env::current_exe()` resolves the actual path
  at runtime. If the user moves the binary, they need to re-run
  `mosaico autostart enable` to update the registry.

## Design Decisions

- **Default No**: the `[y/N]` prompt defaults to No. Auto-starting a
  background daemon should require explicit consent.
- **Registry over Startup folder**: the registry approach is more robust,
  standard, and doesn't require COM interop for shortcut creation.
- **HKCU not HKLM**: per-user startup (no admin rights needed). Each user
  on a shared machine can independently enable/disable Mosaico.
- **Separate `autostart` subcommand**: allows users to manage startup
  independently of `init`. Running `init` again on an existing setup skips
  the prompt (config files already exist), so `autostart enable/disable`
  is the ongoing management interface.
- **`start` argument in registry value**: the registry value includes
  `start` so Windows launches the daemon directly, not the help screen.
- **Skip prompt when configs exist**: if `init` detects all 4 config files
  already exist, the autostart prompt is skipped. This prevents the
  prompt from appearing on repeated `mosaico init` runs.
- **Platform-specific code in `mosaico-windows`**: registry operations are
  Windows-only. Future platform crates can implement their own autostart
  (e.g., systemd user service on Linux, launchd plist on macOS).

## Tasks

- [ ] Create `autostart.rs` in `mosaico-windows/src/` with `enable()`,
      `disable()`, `is_enabled()`, and `exe_path()` using Windows Registry
- [ ] Add `pub mod autostart` to `mosaico-windows/src/lib.rs`
- [ ] Create `autostart.rs` in `mosaico/src/commands/` with `execute()`
      handler for enable/disable/status actions
- [ ] Add `Autostart` subcommand to `main.rs` with `AutostartAction` enum
- [ ] Update `init.rs` to prompt user after config generation:
  - [ ] Print prompt with `[y/N]` default
  - [ ] Read stdin line
  - [ ] Call `autostart::enable()` on "y"/"Y"
  - [ ] Skip prompt if all config files already existed
- [ ] Update `mosaico doctor` to report autostart status (informational)
- [ ] Build with `cargo build`
- [ ] Run `cargo clippy --workspace` and fix warnings
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo test` and fix any failures
- [ ] Manual test: verify registry entry is created/removed, Mosaico starts
      after Windows reboot, `autostart status` reports correctly
- [ ] Update documentation (`docs/cli.md`, `docs/configuration.md`)

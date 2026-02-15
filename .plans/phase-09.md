# Phase 9: Configuration

**Status:** Complete

**Goal:** Load user settings from a TOML config file at `~/.config/mosaico/`.
Configurable values include layout parameters (gap, ratio) and keybindings.
Falls back to sensible defaults when no config file is present.

## Config file location

`~/.config/mosaico/config.toml` — where `~` is the user's home directory.

## Example config

```toml
[layout]
gap = 8
ratio = 0.5

[[keybindings]]
action = "focus-next"
key = "J"
modifiers = ["alt", "shift"]

[[keybindings]]
action = "focus-prev"
key = "K"
modifiers = ["alt", "shift"]

[[keybindings]]
action = "swap-next"
key = "Enter"
modifiers = ["alt", "shift"]

[[keybindings]]
action = "swap-prev"
key = "Enter"
modifiers = ["alt", "ctrl"]

[[keybindings]]
action = "retile"
key = "R"
modifiers = ["alt", "shift"]
```

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    Cargo.toml              # Add toml + dirs dependencies
    src/
      config.rs             # Config, LayoutConfig, Keybinding, Modifier types + load()
      action.rs             # Add serde rename_all = "kebab-case"
      lib.rs                # Re-export config module
  mosaico-windows/
    src/
      keys.rs               # Key name → virtual key code mapping
      hotkey.rs             # register_from_config() using Keybinding config
      event_loop.rs         # Accept keybinding config in start()
      tiling.rs             # Accept BspLayout in TilingManager::new()
      daemon.rs             # Load config, pass to components
      lib.rs                # Declare keys module
  mosaico/
    src/
      commands/
        debug/events.rs     # Update for new start() signature
```

## Tasks

- [x] Add `toml` and `dirs` dependencies to mosaico-core
- [x] Add `#[serde(rename_all = "kebab-case")]` to Action enum
- [x] Create config types (Config, LayoutConfig, Keybinding, Modifier)
- [x] Implement config path resolution and loading
- [x] Create key name → VK code mapping in mosaico-windows
- [x] Add `register_from_config()` to HotkeyManager
- [x] Update event loop to accept keybinding config
- [x] Update TilingManager to accept BspLayout parameter
- [x] Update daemon to load config and pass to components
- [x] Update debug events command for new start() signature
- [x] Build, lint, test
- [x] Commit

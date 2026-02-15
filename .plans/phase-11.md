# Phase 11: Window Rules

**Status:** Complete

**Goal:** Allow users to define rules that control which windows are managed
(tiled) and which are ignored, using class name and title matching.

## Config example

```toml
[[rules]]
match_class = "TaskManagerWindow"
manage = false

[[rules]]
match_title = "Settings"
manage = false
```

## Target structure (new/modified files only)

```
crates/
  mosaico-core/
    src/
      config/
        mod.rs              # Split from config.rs; added WindowRule, should_manage()
        keybinding.rs       # Extracted keybinding types and defaults
  mosaico-windows/
    src/
      tiling.rs             # Added is_tileable() using rules evaluation
      daemon.rs             # Passes rules to TilingManager
```

## Tasks

- [x] Split config.rs into config/ module directory
- [x] Define WindowRule struct (match_class, match_title, manage)
- [x] Implement should_manage() rule evaluation
- [x] Add rules to Config with empty default
- [x] Update TilingManager to accept and use rules
- [x] Update daemon to pass rules from config
- [x] Add unit tests for rule matching
- [x] Build, lint, test
- [x] Commit

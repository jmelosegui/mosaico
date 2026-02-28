# Phase 17: Community Rules Repository

**Status:** In Progress

**Goal:** Create a separate `mosaico-rules` GitHub repository containing
community-maintained default window rules, and update Mosaico to download
these rules on startup. User-specific rules live in a separate file that is
never overwritten, and the two sets are merged at runtime.

## Overview

Today, default rules are hardcoded in `default_rules()` (just
`ApplicationFrameWindow` and `pinentry`). This is too few -- every user
independently discovers that apps like `NVIDIA GeForce Overlay`,
`RealTimeStylus`, `Windows.UI.Core.CoreWindow`, etc. need exclusion rules.

This phase solves the problem by:

1. Creating a `mosaico-rules` repository with a comprehensive, community-curated
   `rules.toml`
2. Having Mosaico download this file on daemon startup and cache it locally
3. Splitting user rules into a separate `user-rules.toml` file
4. Merging both rule sets at runtime (user rules take priority)
5. Encouraging users to contribute rules upstream rather than maintaining
   local overrides

## Reference Design

### File Layout

```
~/.config/mosaico/
  rules.toml          # Downloaded from mosaico-rules repo (overwritten on startup)
  user-rules.toml     # User-specific rules (never overwritten)
```

### Startup Flow

```
mosaico start
  |
  +-> Background thread: download rules.toml from GitHub
  |     |
  |     +-> Success: overwrite ~/.config/mosaico/rules.toml
  |     +-> Failure: log warning, keep existing cached file
  |
  +-> Main thread: load rules.toml + user-rules.toml, merge, continue
```

### Merge Behavior

```
Final rules = user-rules.toml ++ rules.toml
              ^^^^^^^^^^^^^^^    ^^^^^^^^^^
              evaluated first    evaluated second
              (user overrides)   (community defaults)
```

User rules are prepended so they are evaluated first. Since `should_manage()`
returns on the first matching rule, user rules effectively override community
defaults. For example, if the community rules exclude `Chrome_WidgetWin_1`
but a user wants to tile Chrome, they add `manage = true` for that class in
`user-rules.toml`.

## `mosaico-rules` Repository

### Repository Structure

```
mosaico-rules/
  windows/rules.toml   # Windows-specific community rules
  linux/rules.toml     # Linux-specific community rules
  macos/rules.toml     # macOS-specific community rules
  README.md            # Contributing guidelines
  CONTRIBUTING.md      # How to add/test rules
```

### `rules.toml` Format

Same TOML format already used by Mosaico:

```toml
# Community-maintained default rules for Mosaico.
#
# These rules are downloaded automatically when the daemon starts.
# To override a rule, add an entry in ~/.config/mosaico/user-rules.toml
# rather than editing this file (it will be overwritten on next startup).
#
# To contribute: https://github.com/jmelosegui/mosaico-rules

# -- UWP / System --
[[rule]]
match_class = "ApplicationFrameWindow"
manage = false

[[rule]]
match_title = "pinentry"
manage = false

# -- GPU overlays --
[[rule]]
match_class = "NVIDIA GeForce Overlay"
manage = false

[[rule]]
match_title = "AMD Software"
manage = false

# -- System tray / popups --
[[rule]]
match_class = "Windows.UI.Core.CoreWindow"
manage = false

[[rule]]
match_class = "RealTimeStylus"
manage = false

# ... (many more entries contributed over time)
```

### Hosting

The raw file is served via GitHub raw content, with per-OS paths:

```
https://raw.githubusercontent.com/jmelosegui/mosaico-rules/main/windows/rules.toml
https://raw.githubusercontent.com/jmelosegui/mosaico-rules/main/linux/rules.toml
https://raw.githubusercontent.com/jmelosegui/mosaico-rules/main/macos/rules.toml
```

This avoids the GitHub API rate limit (raw.githubusercontent.com has no
rate limit for public repos).

## Architecture

### Modified Files

```
crates/
  mosaico-core/
    src/
      config/
        mod.rs           # Add UserRulesFile, update re-exports
        loader.rs        # Add user_rules_path(), load_user_rules(),
                         #   try_load_user_rules(), cached_rules_path(),
                         #   load_merged_rules()
        template.rs      # Add generate_user_rules() template
  mosaico-windows/
    src/
      config_watcher.rs  # Watch user-rules.toml for hot-reload
      daemon.rs          # Download rules on startup, merge at load time
  mosaico/
    src/
      commands/
        init.rs          # Generate user-rules.toml alongside other files
        doctor.rs        # Validate user-rules.toml
```

### Key Types and Functions

#### `mosaico-core/src/config/loader.rs`

```
user_rules_path()       -> ~/.config/mosaico/user-rules.toml
cached_rules_path()     -> ~/.config/mosaico/rules.toml  (same path, now downloaded)
load_user_rules()       -> Vec<WindowRule>  (silent fallback to empty vec)
try_load_user_rules()   -> Result<Vec<WindowRule>, String>
load_merged_rules()     -> Vec<WindowRule>  (user rules ++ cached rules)
```

`load_merged_rules()` concatenates user rules before community rules so user
rules are evaluated first by `should_manage()`.

#### `mosaico-core/src/config/mod.rs`

- `UserRulesFile` -- serde wrapper for `[[rule]]` arrays in `user-rules.toml`
  (identical structure to `RulesFile`)

#### `mosaico-windows/src/daemon.rs`

On startup, before entering the main loop:

```rust
// Download community rules in the background
let rules_handle = std::thread::spawn(|| {
    download_community_rules()
});

// ... continue with initialization using cached rules ...

// Optionally join the download thread later and reload if new rules arrived
```

`download_community_rules()`:

1. Call `http::get("raw.githubusercontent.com",
   "/jmelosegui/mosaico-rules/main/rules.toml", 5000)`
2. Validate the response as valid TOML (parse as `RulesFile`)
3. If valid, write to `cached_rules_path()`
4. If invalid or network error, log warning and keep existing cached file

#### `mosaico-windows/src/config_watcher.rs`

Add `user-rules.toml` to the watched files. When it changes:

- Send `ConfigReload::Rules(merged_rules)` where `merged_rules` is the
  result of `load_merged_rules()` (user + cached community rules)

The community `rules.toml` is NOT watched for file changes since it is only
updated by the download-on-startup mechanism. However, the watcher reload
for user rules always re-merges both files so it picks up any changes to
the cached community rules too.

### Download Strategy

- **When**: on every daemon startup, in a background thread
- **Timeout**: 5 seconds (non-blocking, daemon starts immediately with
  cached rules)
- **Failure mode**: silent -- log a debug message, use existing cached file.
  If no cached file exists either, fall back to `default_rules()` (the
  hardcoded two rules)
- **Validation**: downloaded content must parse as valid `RulesFile` TOML
  before overwriting the cache. This prevents a corrupted download from
  breaking rules.
- **No version check**: always download the full file. It's small (a few KB)
  and the raw.githubusercontent.com endpoint is fast. Checking ETags or
  timestamps adds complexity with no real benefit.

### Migration from Current `rules.toml`

Today, `rules.toml` contains user-configured rules. After this phase:

- `rules.toml` becomes the community-downloaded file (overwritten on startup)
- User rules move to `user-rules.toml`

Migration path:

1. `mosaico init` generates an empty `user-rules.toml` template
2. If a `rules.toml` already exists and has user modifications beyond the
   default two rules, `init` prints a migration notice telling the user to
   move their custom rules to `user-rules.toml`
3. The first daemon startup downloads the community rules and overwrites
   `rules.toml`

## Configuration

### `user-rules.toml`

```toml
# User-specific window rules for Mosaico.
#
# Rules in this file take priority over community defaults in rules.toml.
# Community rules are downloaded automatically -- to contribute a rule
# that benefits everyone, please submit it to:
#
#   https://github.com/jmelosegui/mosaico-rules
#
# Only add rules here for personal preferences that don't apply to all
# users (e.g., tiling a specific app that most people would exclude).

# Example: force-tile a window that community rules exclude
# [[rule]]
# match_class = "MySpecialApp"
# manage = true

# Example: exclude a personal app
# [[rule]]
# match_title = "My Private Tool"
# manage = false
```

### Hot-Reload

- `user-rules.toml` is hot-reloaded (watched by config watcher)
- `rules.toml` (community) is only updated on daemon startup
- On `user-rules.toml` change, the merged rule set is recomputed and sent
  to the tiling manager via `ConfigReload::Rules`

## Integration Points

- **`mosaico init`**: generates `user-rules.toml` template. Prints migration
  notice if existing `rules.toml` has custom rules.
- **`mosaico doctor`**: validates both `rules.toml` and `user-rules.toml`.
  Reports whether the cached community rules file exists and its age.
- **`mosaico start`**: downloads community rules in background thread before
  or during daemon initialization.
- **Config watcher**: watches `user-rules.toml` (not `rules.toml`). Reload
  merges both files.
- **HTTP client**: reuses the existing `http::get()` in `mosaico-windows`.

## Design Decisions

- **Separate repository**: keeps rules decoupled from the main codebase.
  Rules can be updated independently of Mosaico releases. Community
  contributions don't require building Rust code.
- **Download on startup, not on a timer**: rules change infrequently (days
  or weeks between updates). Downloading once per daemon start is sufficient
  and avoids unnecessary network traffic.
- **Overwrite `rules.toml`, not merge**: the community file is the single
  source of truth. Overwriting ensures users always have the latest rules
  without manual intervention. User customization lives separately.
- **User rules first in merge order**: `should_manage()` evaluates rules
  in order and returns on first match. Prepending user rules gives them
  priority without needing a separate priority field.
- **Silent network failure**: Mosaico must work offline. If the download
  fails, the cached file (or hardcoded defaults) is used. No error is
  shown to the user -- only a debug log entry.
- **Validate before writing**: parsing the downloaded TOML before overwriting
  the cache prevents a partial download or GitHub error page from corrupting
  the rules file.
- **`raw.githubusercontent.com`**: avoids GitHub API rate limits. Public raw
  content has no authentication or rate limit requirements.
- **Encourage upstream contribution**: the generated `user-rules.toml`
  template and documentation both direct users to contribute rules to the
  community repository rather than maintaining local overrides.
- **No `rules.toml` watcher**: the community file is machine-managed
  (downloaded on startup). Watching it for manual edits would be confusing
  since those edits get overwritten. The watcher only watches
  `user-rules.toml`.

## Tasks

### `mosaico-rules` Repository

- [x] Create `mosaico-rules` GitHub repository under `jmelosegui`
- [x] Add `rules.toml` with comprehensive default exclusion rules
  - [x] UWP / system classes (`ApplicationFrameWindow`,
        `Windows.UI.Core.CoreWindow`, etc.)
  - [x] GPU overlays (NVIDIA, AMD, Intel)
  - [x] System utilities (`RealTimeStylus`, `Shell_TrayWnd`, etc.)
  - [ ] Common popups and dialogs
  - [x] GPG/SSH (`pinentry`)
- [ ] Add `README.md` with project description and usage
- [ ] Add `CONTRIBUTING.md` with instructions for testing and submitting rules

### Mosaico Changes

- [x] Add `user_rules_path()` to `loader.rs`
      (`~/.config/mosaico/user-rules.toml`)
- [x] Add `load_user_rules()` and `try_load_user_rules()` to `loader.rs`
- [x] Add `load_merged_rules()` to `loader.rs` (user ++ cached community)
- [x] Add `UserRulesFile` serde wrapper to `config/mod.rs`
- [x] Update `re-exports` in `config/mod.rs` for new loader functions
- [x] Add `generate_user_rules()` to `template.rs` with commented template
- [x] Update `init.rs` to generate `user-rules.toml`
- [ ] Update `init.rs` to print migration notice if `rules.toml` has custom
      rules beyond defaults
- [x] Add `download_community_rules()` function in daemon (uses `http::get`)
- [x] Spawn background download thread in `daemon_loop()` before main loop
- [x] Update daemon to call `load_merged_rules()` instead of `load_rules()`
- [x] Update `config_watcher.rs` to watch `user-rules.toml` instead of (or
      in addition to) `rules.toml`
- [x] Update watcher reload to use `load_merged_rules()` for
      `ConfigReload::Rules`
- [x] Update `doctor.rs` to validate both `rules.toml` and `user-rules.toml`
- [ ] Update `doctor.rs` to report community rules cache age
- [x] Build with `cargo build`
- [x] Run `cargo clippy --workspace` and fix warnings
- [x] Run `cargo fmt --all`
- [x] Run `cargo test` and fix any failures
- [ ] Update documentation (`docs/configuration.md`, `docs/window-management.md`)
